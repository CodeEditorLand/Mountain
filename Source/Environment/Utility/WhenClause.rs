//! VS Code context-key "when" clause parser and evaluator.
//!
//! A when clause is a boolean expression over context keys that gates
//! keybindings, menu items, and view visibility. The grammar implemented
//! here matches `vs/platform/contextkey/common/contextkey.ts`:
//!
//! - `||` (lowest precedence), `&&`, unary `!`, parentheses
//! - Comparisons: `==`, `!=`, `<`, `<=`, `>`, `>=`
//! - Regex match: `key =~ /pattern/flags` (only the `i` flag is honoured)
//! - List membership: `key in otherKey`, `key not in otherKey`
//! - Literals: `true`, `false`, `'single-quoted'`, `"double-quoted"`,
//!   numbers, and bare words
//!
//! Evaluation runs against a JSON object snapshot of context keys supplied
//! by the caller (Sky owns the live context; Mountain receives snapshots
//! over IPC). A missing key evaluates as `undefined`: falsy on its own,
//! unequal to everything but `''`/`false` comparisons mirror VS Code's
//! loose-string semantics.
//!
//! Also hosts the keybinding helpers built on top of the evaluator:
//! key-expression normalisation (modifier aliasing + canonical ordering,
//! chord-aware) and when-clause specificity scoring used for conflict
//! precedence (more specific clause wins at equal source weight).

use serde_json::Value;

/// Parsed form of a when clause.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum WhenExpression {
	True,

	False,

	/// Bare context key: truthy test.
	Defined(String),

	Not(Box<WhenExpression>),

	And(Vec<WhenExpression>),

	Or(Vec<WhenExpression>),

	/// `key == literal` (string-coerced comparison).
	Equals(String, String),

	/// `key != literal`.
	NotEquals(String, String),

	/// `key =~ /pattern/flags`.
	Matches(String, String, bool),

	/// `key < n`, `key <= n`, `key > n`, `key >= n`.
	Compare(String, CompareOperator, f64),

	/// `key in otherKey` / `key not in otherKey`.
	In(String, String, bool),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum CompareOperator {
	Less,

	LessOrEqual,

	Greater,

	GreaterOrEqual,
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
	Identifier(String),

	StringLiteral(String),

	Regex(String, bool),

	Number(f64),

	Bang,

	AndAnd,

	OrOr,

	EqualsEquals,

	NotEquals,

	MatchesOperator,

	Less,

	LessOrEqual,

	Greater,

	GreaterOrEqual,

	In,

	NotKeyword,

	OpenParen,

	CloseParen,
}

/// Tokenize a when-clause source string. The regex literal form is only
/// valid immediately after `=~`, which the tokenizer tracks with a small
/// mode flag (mirrors the scanner in VS Code's contextkey parser).
fn Tokenize(Source:&str) -> Result<Vec<Token>, String> {
	let mut Tokens = Vec::new();

	let Bytes:Vec<char> = Source.chars().collect();

	let mut Index = 0usize;

	let mut ExpectRegex = false;

	while Index < Bytes.len() {
		let Character = Bytes[Index];

		match Character {
			' ' | '\t' | '\r' | '\n' => {
				Index += 1;
			},
			'(' => {
				Tokens.push(Token::OpenParen);

				Index += 1;
			},
			')' => {
				Tokens.push(Token::CloseParen);

				Index += 1;
			},
			'!' => {
				if Bytes.get(Index + 1) == Some(&'=') {
					Tokens.push(Token::NotEquals);

					Index += 2;
				} else {
					Tokens.push(Token::Bang);

					Index += 1;
				}
			},
			'&' => {
				if Bytes.get(Index + 1) == Some(&'&') {
					Tokens.push(Token::AndAnd);

					Index += 2;
				} else {
					return Err(format!("Unexpected '&' at offset {} in when clause", Index));
				}
			},
			'|' => {
				if Bytes.get(Index + 1) == Some(&'|') {
					Tokens.push(Token::OrOr);

					Index += 2;
				} else {
					return Err(format!("Unexpected '|' at offset {} in when clause", Index));
				}
			},
			'=' => {
				if Bytes.get(Index + 1) == Some(&'=') {
					Tokens.push(Token::EqualsEquals);

					Index += 2;
				} else if Bytes.get(Index + 1) == Some(&'~') {
					Tokens.push(Token::MatchesOperator);

					ExpectRegex = true;

					Index += 2;
				} else {
					return Err(format!("Unexpected '=' at offset {} in when clause", Index));
				}
			},
			'<' => {
				if Bytes.get(Index + 1) == Some(&'=') {
					Tokens.push(Token::LessOrEqual);

					Index += 2;
				} else {
					Tokens.push(Token::Less);

					Index += 1;
				}
			},
			'>' => {
				if Bytes.get(Index + 1) == Some(&'=') {
					Tokens.push(Token::GreaterOrEqual);

					Index += 2;
				} else {
					Tokens.push(Token::Greater);

					Index += 1;
				}
			},
			'\'' | '"' => {
				let Quote = Character;

				let Start = Index + 1;

				let mut End = Start;

				while End < Bytes.len() && Bytes[End] != Quote {
					End += 1;
				}

				if End >= Bytes.len() {
					return Err(format!("Unterminated string literal at offset {} in when clause", Index));
				}

				Tokens.push(Token::StringLiteral(Bytes[Start..End].iter().collect()));

				Index = End + 1;
			},
			'/' if ExpectRegex => {
				let Start = Index + 1;

				let mut End = Start;

				while End < Bytes.len() && Bytes[End] != '/' {
					if Bytes[End] == '\\' {
						End += 1;
					}

					End += 1;
				}

				if End >= Bytes.len() {
					return Err(format!("Unterminated regex literal at offset {} in when clause", Index));
				}

				let Pattern:String = Bytes[Start..End].iter().collect();

				let mut FlagEnd = End + 1;

				let mut CaseInsensitive = false;

				while FlagEnd < Bytes.len() && Bytes[FlagEnd].is_ascii_alphabetic() {
					if Bytes[FlagEnd] == 'i' {
						CaseInsensitive = true;
					}

					FlagEnd += 1;
				}

				Tokens.push(Token::Regex(Pattern, CaseInsensitive));

				ExpectRegex = false;

				Index = FlagEnd;
			},
			_ => {
				if Character.is_ascii_digit()
					|| (Character == '-' && Bytes.get(Index + 1).is_some_and(|C| C.is_ascii_digit()))
				{
					let Start = Index;

					let mut End = Index + 1;

					while End < Bytes.len() && (Bytes[End].is_ascii_digit() || Bytes[End] == '.') {
						End += 1;
					}

					let Text:String = Bytes[Start..End].iter().collect();

					let Parsed = Text
						.parse::<f64>()
						.map_err(|_| format!("Invalid number '{}' in when clause", Text))?;

					Tokens.push(Token::Number(Parsed));

					Index = End;
				} else if Character.is_alphanumeric() || Character == '_' || Character == '.' || Character == '-' {
					let Start = Index;

					let mut End = Index;

					while End < Bytes.len()
						&& (Bytes[End].is_alphanumeric()
							|| Bytes[End] == '_' || Bytes[End] == '.'
							|| Bytes[End] == '-' || Bytes[End] == ':'
							|| Bytes[End] == '/')
					{
						End += 1;
					}

					let Word:String = Bytes[Start..End].iter().collect();

					match Word.as_str() {
						"in" => Tokens.push(Token::In),
						"not" => Tokens.push(Token::NotKeyword),
						_ => Tokens.push(Token::Identifier(Word)),
					}

					Index = End;
				} else {
					return Err(format!("Unexpected character '{}' at offset {} in when clause", Character, Index));
				}
			},
		}
	}

	Ok(Tokens)
}

struct Parser {
	Tokens:Vec<Token>,

	Position:usize,
}

impl Parser {
	fn Peek(&self) -> Option<&Token> { self.Tokens.get(self.Position) }

	fn Advance(&mut self) -> Option<Token> {
		let Token = self.Tokens.get(self.Position).cloned();

		self.Position += 1;

		Token
	}

	/// `orExpr := andExpr ('||' andExpr)*`
	fn ParseOr(&mut self) -> Result<WhenExpression, String> {
		let mut Parts = vec![self.ParseAnd()?];

		while self.Peek() == Some(&Token::OrOr) {
			self.Advance();

			Parts.push(self.ParseAnd()?);
		}

		if Parts.len() == 1 { Ok(Parts.remove(0)) } else { Ok(WhenExpression::Or(Parts)) }
	}

	/// `andExpr := unary ('&&' unary)*`
	fn ParseAnd(&mut self) -> Result<WhenExpression, String> {
		let mut Parts = vec![self.ParseUnary()?];

		while self.Peek() == Some(&Token::AndAnd) {
			self.Advance();

			Parts.push(self.ParseUnary()?);
		}

		if Parts.len() == 1 { Ok(Parts.remove(0)) } else { Ok(WhenExpression::And(Parts)) }
	}

	/// `unary := '!' unary | primary`
	fn ParseUnary(&mut self) -> Result<WhenExpression, String> {
		if self.Peek() == Some(&Token::Bang) {
			self.Advance();

			return Ok(WhenExpression::Not(Box::new(self.ParseUnary()?)));
		}

		self.ParsePrimary()
	}

	/// `primary := '(' orExpr ')' | 'true' | 'false' | key (operator operand)?`
	fn ParsePrimary(&mut self) -> Result<WhenExpression, String> {
		match self.Advance() {
			Some(Token::OpenParen) => {
				let Inner = self.ParseOr()?;

				if self.Advance() != Some(Token::CloseParen) {
					return Err("Expected ')' in when clause".to_string());
				}

				Ok(Inner)
			},
			Some(Token::Identifier(Key)) => {
				match Key.as_str() {
					"true" => return Ok(WhenExpression::True),
					"false" => return Ok(WhenExpression::False),
					_ => {},
				}

				match self.Peek() {
					Some(Token::EqualsEquals) => {
						self.Advance();

						Ok(WhenExpression::Equals(Key, self.ParseOperand()?))
					},
					Some(Token::NotEquals) => {
						self.Advance();

						Ok(WhenExpression::NotEquals(Key, self.ParseOperand()?))
					},
					Some(Token::MatchesOperator) => {
						self.Advance();

						match self.Advance() {
							Some(Token::Regex(Pattern, CaseInsensitive)) => {
								Ok(WhenExpression::Matches(Key, Pattern, CaseInsensitive))
							},
							Some(Token::StringLiteral(Pattern)) => Ok(WhenExpression::Matches(Key, Pattern, false)),
							_ => Err("Expected regex after '=~' in when clause".to_string()),
						}
					},
					Some(Token::Less) => {
						self.Advance();

						Ok(WhenExpression::Compare(Key, CompareOperator::Less, self.ParseNumber()?))
					},
					Some(Token::LessOrEqual) => {
						self.Advance();

						Ok(WhenExpression::Compare(Key, CompareOperator::LessOrEqual, self.ParseNumber()?))
					},
					Some(Token::Greater) => {
						self.Advance();

						Ok(WhenExpression::Compare(Key, CompareOperator::Greater, self.ParseNumber()?))
					},
					Some(Token::GreaterOrEqual) => {
						self.Advance();

						Ok(WhenExpression::Compare(Key, CompareOperator::GreaterOrEqual, self.ParseNumber()?))
					},
					Some(Token::In) => {
						self.Advance();

						match self.Advance() {
							Some(Token::Identifier(ListKey)) => Ok(WhenExpression::In(Key, ListKey, false)),
							_ => Err("Expected context key after 'in' in when clause".to_string()),
						}
					},
					Some(Token::NotKeyword) => {
						self.Advance();

						if self.Advance() != Some(Token::In) {
							return Err("Expected 'in' after 'not' in when clause".to_string());
						}

						match self.Advance() {
							Some(Token::Identifier(ListKey)) => Ok(WhenExpression::In(Key, ListKey, true)),
							_ => Err("Expected context key after 'not in' in when clause".to_string()),
						}
					},
					_ => Ok(WhenExpression::Defined(Key)),
				}
			},
			Other => Err(format!("Unexpected token {:?} in when clause", Other)),
		}
	}

	/// Comparison operands are string-coerced: quoted strings, bare words,
	/// numbers, and booleans all compare via their string form.
	fn ParseOperand(&mut self) -> Result<String, String> {
		match self.Advance() {
			Some(Token::StringLiteral(Text)) => Ok(Text),
			Some(Token::Identifier(Word)) => Ok(Word),
			Some(Token::Number(Number)) => {
				if Number.fract() == 0.0 {
					Ok(format!("{}", Number as i64))
				} else {
					Ok(format!("{}", Number))
				}
			},
			Other => Err(format!("Expected literal after comparison, found {:?}", Other)),
		}
	}

	fn ParseNumber(&mut self) -> Result<f64, String> {
		match self.Advance() {
			Some(Token::Number(Number)) => Ok(Number),
			Some(Token::StringLiteral(Text)) | Some(Token::Identifier(Text)) => {
				Text.parse::<f64>().map_err(|_| format!("Expected number, found '{}' in when clause", Text))
			},
			Other => Err(format!("Expected number after comparison, found {:?}", Other)),
		}
	}
}

/// Parse a when-clause string into its expression tree.
pub(crate) fn Parse(Source:&str) -> Result<WhenExpression, String> {
	let Trimmed = Source.trim();

	if Trimmed.is_empty() {
		return Ok(WhenExpression::True);
	}

	let Tokens = Tokenize(Trimmed)?;

	let mut Machine = Parser { Tokens, Position:0 };

	let Expression = Machine.ParseOr()?;

	if Machine.Position != Machine.Tokens.len() {
		return Err(format!("Trailing tokens after expression in when clause '{}'", Trimmed));
	}

	Ok(Expression)
}

/// Coerce a context value to its string form for `==`/`!=` comparisons
/// (VS Code compares loosely against the literal's source text).
fn ValueToComparableString(ContextValue:&Value) -> String {
	match ContextValue {
		Value::String(Text) => Text.clone(),
		Value::Bool(Flag) => Flag.to_string(),
		Value::Number(Number) => Number.to_string(),
		Value::Null => "undefined".to_string(),
		Other => Other.to_string(),
	}
}

/// JavaScript-style truthiness for a bare context key.
fn IsTruthy(ContextValue:Option<&Value>) -> bool {
	match ContextValue {
		None | Some(Value::Null) => false,
		Some(Value::Bool(Flag)) => *Flag,
		Some(Value::String(Text)) => !Text.is_empty(),
		Some(Value::Number(Number)) => Number.as_f64().is_some_and(|N| N != 0.0),
		Some(Value::Array(_)) | Some(Value::Object(_)) => true,
	}
}

/// Evaluate a parsed expression against a context-key snapshot
/// (a JSON object mapping key → value). Missing keys are `undefined`.
pub(crate) fn Evaluate(Expression:&WhenExpression, Context:&Value) -> bool {
	let Lookup = |Key:&str| -> Option<&Value> { Context.get(Key) };

	match Expression {
		WhenExpression::True => true,
		WhenExpression::False => false,
		WhenExpression::Defined(Key) => IsTruthy(Lookup(Key)),
		WhenExpression::Not(Inner) => !Evaluate(Inner, Context),
		WhenExpression::And(Parts) => Parts.iter().all(|Part| Evaluate(Part, Context)),
		WhenExpression::Or(Parts) => Parts.iter().any(|Part| Evaluate(Part, Context)),
		WhenExpression::Equals(Key, Literal) => {
			match Lookup(Key) {
				None => Literal == "undefined" || Literal == "false" || Literal.is_empty(),
				Some(ContextValue) => ValueToComparableString(ContextValue) == *Literal,
			}
		},
		WhenExpression::NotEquals(Key, Literal) => {
			match Lookup(Key) {
				None => !(Literal == "undefined" || Literal == "false" || Literal.is_empty()),
				Some(ContextValue) => ValueToComparableString(ContextValue) != *Literal,
			}
		},
		WhenExpression::Matches(Key, Pattern, CaseInsensitive) => {
			let Some(ContextValue) = Lookup(Key) else {
				return false;
			};

			let Subject = ValueToComparableString(ContextValue);

			let Source = if *CaseInsensitive { format!("(?i){}", Pattern) } else { Pattern.clone() };

			match regex::Regex::new(&Source) {
				Ok(Expression) => Expression.is_match(&Subject),
				Err(_) => false,
			}
		},
		WhenExpression::Compare(Key, Operator, Operand) => {
			let Number = match Lookup(Key) {
				Some(Value::Number(N)) => N.as_f64(),
				Some(Value::String(Text)) => Text.parse::<f64>().ok(),
				_ => None,
			};

			let Some(Left) = Number else {
				return false;
			};

			match Operator {
				CompareOperator::Less => Left < *Operand,
				CompareOperator::LessOrEqual => Left <= *Operand,
				CompareOperator::Greater => Left > *Operand,
				CompareOperator::GreaterOrEqual => Left >= *Operand,
			}
		},
		WhenExpression::In(Key, ListKey, Negated) => {
			let Needle = Lookup(Key).map(ValueToComparableString).unwrap_or_else(|| Key.clone());

			let Contained = match Lookup(ListKey) {
				Some(Value::Array(Items)) => Items.iter().any(|Item| ValueToComparableString(Item) == Needle),
				Some(Value::Object(Map)) => Map.contains_key(&Needle),
				Some(Value::String(Text)) => Text.contains(&Needle),
				_ => false,
			};

			if *Negated { !Contained } else { Contained }
		},
	}
}

/// Parse + evaluate in one step. An unparseable clause deactivates the
/// guarded item (`false`), matching VS Code's behaviour for invalid
/// expressions; an absent clause activates it (`true`).
pub(crate) fn EvaluateClause(Clause:Option<&str>, Context:&Value) -> bool {
	match Clause {
		None => true,
		Some(Source) => {
			match Parse(Source) {
				Ok(Expression) => Evaluate(&Expression, Context),
				Err(_) => false,
			}
		},
	}
}

/// Count the leaf terms of a clause - the precedence score used to break
/// conflicts between bindings of equal source weight (a binding guarded
/// by `editorTextFocus && !inQuickOpen` beats one guarded by
/// `editorTextFocus` which beats one with no clause).
pub(crate) fn SpecificityOf(Expression:&WhenExpression) -> u32 {
	match Expression {
		WhenExpression::True | WhenExpression::False => 0,
		WhenExpression::Not(Inner) => SpecificityOf(Inner),
		WhenExpression::And(Parts) | WhenExpression::Or(Parts) => Parts.iter().map(SpecificityOf).sum(),
		_ => 1,
	}
}

/// Normalise a key expression for equality comparison: lowercase, alias
/// modifiers to canonical names (`cmd`, `ctrl`, `shift`, `alt`), order
/// modifiers canonically within each stroke, and separate chord strokes
/// with single spaces. `"Shift+CMD+p"` and `"meta+shift+P"` both
/// normalise to `"cmd+shift+p"`.
pub(crate) fn NormalizeKeyExpression(Expression:&str) -> String {
	Expression
		.split_whitespace()
		.map(|Stroke| {
			let mut Modifiers:Vec<&str> = Vec::new();

			let mut BaseKey = String::new();

			for Part in Stroke.split('+') {
				let Lower = Part.trim().to_lowercase();

				match Lower.as_str() {
					"ctrl" | "control" => Modifiers.push("ctrl"),
					"shift" => Modifiers.push("shift"),
					"alt" | "option" | "opt" => Modifiers.push("alt"),
					"cmd" | "command" | "meta" | "super" | "win" => Modifiers.push("cmd"),
					"" => {},
					_ => BaseKey = Lower,
				}
			}

			let Order = |Name:&&str| {
				match *Name {
					"ctrl" => 0,
					"shift" => 1,
					"alt" => 2,
					_ => 3,
				}
			};

			Modifiers.sort_by_key(Order);

			Modifiers.dedup();

			if BaseKey.is_empty() {
				Modifiers.join("+")
			} else if Modifiers.is_empty() {
				BaseKey
			} else {
				format!("{}+{}", Modifiers.join("+"), BaseKey)
			}
		})
		.collect::<Vec<String>>()
		.join(" ")
}

#[cfg(test)]
mod Tests {
	use serde_json::json;

	use super::*;

	#[test]
	fn BareKeyTruthiness() {
		let Context = json!({ "editorTextFocus": true, "emptyText": "", "count": 0 });

		assert!(EvaluateClause(Some("editorTextFocus"), &Context));

		assert!(!EvaluateClause(Some("emptyText"), &Context));

		assert!(!EvaluateClause(Some("count"), &Context));

		assert!(!EvaluateClause(Some("missingKey"), &Context));
	}

	#[test]
	fn BooleanOperatorsAndPrecedence() {
		let Context = json!({ "a": true, "b": false, "c": true });

		assert!(EvaluateClause(Some("a && c"), &Context));

		assert!(!EvaluateClause(Some("a && b"), &Context));

		// && binds tighter than ||: b && b || a == (b && b) || a
		assert!(EvaluateClause(Some("b && b || a"), &Context));

		assert!(!EvaluateClause(Some("b && (b || a)"), &Context));

		assert!(EvaluateClause(Some("!b"), &Context));
	}

	#[test]
	fn EqualityComparisons() {
		let Context = json!({ "resourceLangId": "python", "debugState": "inactive", "groupCount": 2 });

		assert!(EvaluateClause(Some("resourceLangId == python"), &Context));

		assert!(EvaluateClause(Some("resourceLangId == 'python'"), &Context));

		assert!(EvaluateClause(Some("debugState != 'running'"), &Context));

		assert!(EvaluateClause(Some("groupCount == 2"), &Context));

		assert!(!EvaluateClause(Some("resourceLangId == rust"), &Context));
	}

	#[test]
	fn NumericComparisons() {
		let Context = json!({ "groupCount": 3 });

		assert!(EvaluateClause(Some("groupCount > 2"), &Context));

		assert!(EvaluateClause(Some("groupCount >= 3"), &Context));

		assert!(!EvaluateClause(Some("groupCount < 3"), &Context));

		assert!(!EvaluateClause(Some("missing > 0"), &Context));
	}

	#[test]
	fn RegexMatching() {
		let Context = json!({ "resourceFilename": "Main.RS" });

		assert!(EvaluateClause(Some("resourceFilename =~ /\\.rs$/i"), &Context));

		assert!(!EvaluateClause(Some("resourceFilename =~ /\\.ts$/"), &Context));
	}

	#[test]
	fn ListMembership() {
		let Context =
			json!({ "resourceExtname": ".rs", "supportedExtensions": [".rs", ".toml"], "viewMap": { "a": 1 } });

		assert!(EvaluateClause(Some("resourceExtname in supportedExtensions"), &Context));

		assert!(!EvaluateClause(Some("resourceExtname not in supportedExtensions"), &Context));

		assert!(EvaluateClause(Some("a in viewMap"), &Context));
	}

	#[test]
	fn InvalidAndEmptyClauses() {
		let Context = json!({});

		assert!(EvaluateClause(None, &Context));

		assert!(EvaluateClause(Some("   "), &Context));

		assert!(!EvaluateClause(Some("a &&"), &Context));

		assert!(!EvaluateClause(Some("a & b"), &Context));
	}

	#[test]
	fn SpecificityScoring() {
		assert_eq!(SpecificityOf(&Parse("editorTextFocus && !inQuickOpen").unwrap()), 2);

		assert_eq!(SpecificityOf(&Parse("editorTextFocus").unwrap()), 1);

		assert_eq!(SpecificityOf(&Parse("").unwrap()), 0);
	}

	#[test]
	fn KeyNormalisation() {
		assert_eq!(NormalizeKeyExpression("Shift+CMD+p"), "cmd+shift+p");

		assert_eq!(NormalizeKeyExpression("meta+shift+P"), "cmd+shift+p");

		assert_eq!(NormalizeKeyExpression("Ctrl+K  Ctrl+C"), "ctrl+k ctrl+c");

		assert_eq!(NormalizeKeyExpression("option+Tab"), "alt+tab");
	}
}
