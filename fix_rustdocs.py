#!/usr/bin/env python3
"""
Rewrite all rustdoc comments in Mountain modules to meet quality standards.

Run from Land/Element/Mountain/:
    python3 fix_rustdocs.py
    cargo check -p mountain 2>&1 | grep -E "warning|error" | head -10
"""

import os
import re
import glob
import shutil

SOURCE_DIR = os.path.join(
    os.path.dirname(os.path.abspath(__file__)),
    "Source"
)

TARGET_MODULES = [
    "Vine",
    "ApplicationState",
    "Command",
    "Track",
    "ProcessManagement",
    "Telemetry",
    "Air",
    "RunTime",
    "Update",
    "Error",
    "ExtensionManagement",
    "Workspace",
    "Cache",
    "FileSystem",
]

# ─── PASCAL CASE SPLIT ─────────────────────────────

def split_pascal(name):
    """Split PascalCase into words: 'MarkerDataDTO' -> ['Marker', 'Data', 'DTO']."""
    # Handle consecutive uppercase (DTO, URI, IPC, etc.)
    parts = []
    for match in re.finditer(r'[A-Z][a-z]+|[A-Z]+(?=[A-Z]|$)|[A-Z][a-z]*', name):
        parts.append(match.group(0))
    if not parts:
        parts = [name]
    return parts

def pascal_to_desc(name):
    """Convert PascalCase name to a human-readable description."""
    words = split_pascal(name)
    return ' '.join(w.lower() for w in words)

def pascal_to_verb_phrase(name):
    """Convert a PascalCase verb to third-person: 'ApplyUpdate' -> 'Applies an update'."""
    words = split_pascal(name)
    if not words:
        return name.lower()
    
    verb = words[0].lower()
    # Basic English third-person rules
    if verb.endswith('s') or verb.endswith('sh') or verb.endswith('ch') or verb.endswith('x') or verb.endswith('z'):
        verb = verb + 'es'
    elif verb.endswith('y') and len(verb) > 1 and verb[-2] not in 'aeiou':
        verb = verb[:-1] + 'ies'
    else:
        verb = verb + 's'
    
    rest = ' '.join(w.lower() for w in words[1:])
    article = 'an' if rest and rest[0] in 'aeiou' else 'a'
    if rest:
        return f"{verb} {article} {rest}"
    return verb.capitalize()

# ─── META PATTERNS ─────────────────────────────────

META_REPLACEMENTS = [
    # "This function does X" → "Does X"
    (re.compile(r'^/// This function (does|builds|creates|returns|sends|receives|handles|processes|manages|validates|checks|resolves|loads|parses|queries|updates|deletes|starts|stops|provides|performs|sets|gets|runs|executes|generates|extracts|merges|inserts|removes|closes|opens|writes|reads|configures|finds|collects|enables|disables|registers|unregisters|refreshes|restores|persists|dispatches|cancels|tracks|monitors|launches|spawns|cleans|sweeps|seeds) '),
     lambda m: f'/// {m.group(1).capitalize()} '),
    # "This struct/enum/trait/module/type..." → remove "This"
    (re.compile(r'^/// This (struct|enum|trait|module|type|macro|const|static|method|class) '),
     lambda m: '/// '),
    # "Represents " → remove
    (re.compile(r'^/// Represents '),
     lambda m: '/// '),
    # "This function is responsible for " → remove
    (re.compile(r'^/// This function is responsible for '),
     lambda m: '/// '),
    # "This module is responsible for " → remove
    (re.compile(r'^/// This module is responsible for '),
     lambda m: '/// '),
    # "This is used to " → remove
    (re.compile(r'^/// This is used to '),
     lambda m: '/// '),
]

def fix_meta_instructional(line):
    """Fix '/// This function...' patterns by removing the meta wrapper."""
    for pattern, repl_func in META_REPLACEMENTS:
        if pattern.search(line):
            return pattern.sub(repl_func, line)
    return line

# ─── PUB ITEM DETECTION ──────────────────────────

def get_pub_items(lines):
    """Find pub items and their preceding doc/attr lines."""
    pub_items = []
    for i, line in enumerate(lines):
        stripped = line.strip()
        if not stripped.startswith("pub "):
            continue
        
        # Collect preceding doc comments
        doc_lines = []
        j = i - 1
        while j >= 0:
            prev = lines[j].strip()
            if prev.startswith("///"):
                doc_lines.insert(0, prev)
            elif prev.startswith("#[") or prev.startswith("#!["):
                pass  # skip attributes
            elif prev == "":
                break
            else:
                break
            j -= 1
        
        item_type, item_name = detect_item_type_name(stripped)
        if item_type:
            pub_items.append({
                'line': i,
                'type': item_type,
                'name': item_name,
                'has_doc': len(doc_lines) > 0,
            })
    return pub_items

def detect_item_type_name(stripped):
    """Extract (type, name) from a pub line."""
    # pub struct Foo
    m = re.match(r'pub\s+(struct|enum|trait|type|mod)\s+([a-zA-Z_][a-zA-Z0-9_]*)', stripped)
    if m:
        return m.group(1), m.group(2)
    # pub const FOO
    m = re.match(r'pub\s+const\s+([a-zA-Z_][a-zA-Z0-9_]*)', stripped)
    if m:
        return 'const', m.group(1)
    # pub static FOO
    m = re.match(r'pub\s+static\s+([a-zA-Z_][a-zA-Z0-9_]*)', stripped)
    if m:
        return 'static', m.group(1)
    # pub fn foo or pub async fn foo
    m = re.match(r'pub\s+(?:async\s+)?(?:unsafe\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)', stripped)
    if m:
        return 'fn', m.group(1)
    # pub macro_rules!
    m = re.match(r'pub\s+macro_rules!\s*([a-zA-Z_][a-zA-Z0-9_]*)', stripped)
    if m:
        return 'macro', m.group(1)
    return None, None

def gen_fn_doc(name):
    """Generate a proper doc comment for a function."""
    if not name or name == 'Fn':
        return None  # generic wrapper
    
    # All-lowercase function name (snake_case or plain)
    if name.islower() or '_' in name:
        desc = name.replace('_', ' ').capitalize()
        if not desc.endswith('.'):
            desc += '.'
        return f'/// {desc}'
    
    # PascalCase function name - these are our "action" functions
    # Handle common verb endings properly
    desc = pascal_to_verb_phrase(name)
    if not desc.endswith('.'):
        desc += '.'
    return f'/// {desc}'

def gen_item_doc(item_type, name):
    """Generate doc comment for a pub item."""
    if not name:
        return None
    
    if item_type == 'fn':
        return gen_fn_doc(name)
    
    if item_type == 'struct':
        words = split_pascal(name)
        desc = ' '.join(w.lower() for w in words)
        return f'/// {desc.capitalize()}.'
    
    if item_type == 'enum':
        words = split_pascal(name)
        desc = ' '.join(w.lower() for w in words)
        return f'/// Enumeration of {desc}.'
    
    if item_type == 'trait':
        words = split_pascal(name)
        desc = ' '.join(w.lower() for w in words)
        return f'/// Trait for {desc}.'
    
    if item_type == 'type':
        desc = name.replace('_', ' ').capitalize()
        return f'/// Type alias for {desc.lower()}.'
    
    if item_type in ('const', 'static'):
        desc = name.replace('_', ' ').lower()
        return f'/// {desc.capitalize()}.'
    
    if item_type == 'mod':
        desc = name.replace('_', ' ').capitalize()
        return f'/// {desc} module.'
    
    if item_type == 'macro':
        return f'/// Macro definition.'
    
    return None

# ─── FILE PROCESSING ──────────────────────────────

def fix_doc_text(text):
    """Apply all doc-text quality fixes to a single line."""
    # Fix meta-instructional text
    text = fix_meta_instructional(text)
    return text

def process_file(filepath):
    """Process a single .rs file."""
    with open(filepath, 'r', encoding='utf-8') as f:
        original = f.read()
    
    lines = original.splitlines(keepends=True)
    
    # Stage 1: Fix doc comment text content
    for i in range(len(lines)):
        stripped = lines[i].strip()
        if stripped.startswith("///") and not stripped.startswith("////"):
            lines[i] = fix_doc_text(lines[i])
        elif stripped.startswith("//!"):
            lines[i] = fix_doc_text(lines[i])
    
    # Stage 2: Remove empty doc-comment lines (just "///")
    new_lines = []
    for line in lines:
        stripped = line.strip()
        if stripped == "///":
            continue
        new_lines.append(line)
    
    # Stage 3: Collapse consecutive empty lines to at most one
    collapsed = []
    prev_empty = False
    for line in new_lines:
        is_empty = line.strip() == ""
        if is_empty and prev_empty:
            continue
        collapsed.append(line)
        prev_empty = is_empty
    
    lines = collapsed
    
    # Stage 4: Add missing docs on pub items
    pub_items = get_pub_items(lines)
    for item in reversed(pub_items):
        if not item['has_doc']:
            doc = gen_item_doc(item['type'], item['name'])
            if doc:
                lines.insert(item['line'], doc + '\n')
    
    content = ''.join(lines)
    
    if content != original:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)
        return True
    return False

def is_generated(filepath):
    """Skip auto-generated files."""
    name = os.path.basename(filepath)
    return name in ("vine.rs",) or "/Generated/" in filepath

def should_process(filepath):
    """Check if file is in target modules."""
    rel = os.path.relpath(filepath, SOURCE_DIR)
    for module in TARGET_MODULES:
        if rel.startswith(module) or rel == f"{module}.rs":
            return True
    if os.path.basename(filepath) == "Library.rs":
        return True
    return False

def main():
    all_files = []
    for root, dirs, files in os.walk(SOURCE_DIR):
        for f in files:
            if f.endswith('.rs'):
                filepath = os.path.join(root, f)
                if should_process(filepath):
                    all_files.append(filepath)
    
    library_rs = os.path.join(SOURCE_DIR, "Library.rs")
    if os.path.exists(library_rs):
        if library_rs not in all_files:
            all_files.append(library_rs)
    
    all_files = sorted(set(all_files))
    print(f"Found {len(all_files)} .rs files to process")
    
    changed = 0
    skipped = 0
    
    for fp in all_files:
        if is_generated(fp):
            skipped += 1
            continue
        if process_file(fp):
            changed += 1
            rel = os.path.relpath(fp, SOURCE_DIR)
            print(f"  ✓ {rel}")
    
    total = len(all_files)
    unchanged = total - changed - skipped
    print(f"\nDone: {changed} files modified, {unchanged} unchanged, {skipped} generated skipped")

if __name__ == "__main__":
    main()
