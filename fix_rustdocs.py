#!/usr/bin/env python3
"""
Rewrite all rustdoc comments in Mountain modules to meet quality standards.

Run from Land/Element/Mountain/:
    python3 fix_rustdocs.py
    cargo check -p mountain 2>&1 | grep -E "warning|error" | head -10
"""

import os
import re

SOURCE_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "Source")

TARGET_MODULES = [
    "Vine", "ApplicationState", "Command", "Track", "ProcessManagement",
    "Telemetry", "Air", "RunTime", "Update", "Error", "ExtensionManagement",
    "Workspace", "Cache", "FileSystem",
]

# ─── PASCAL CASE UTILITIES ─────────────────────────

def split_pascal(name):
    """Split PascalCase into words: 'MarkerDataDTO' -> ['Marker', 'Data', 'DTO']."""
    parts = []
    for match in re.finditer(r'[A-Z][a-z]+|[A-Z]+(?=[A-Z][a-z]|$)|[A-Z]+', name):
        parts.append(match.group(0))
    return parts or [name]

def pascal_to_desc(name):
    """PascalCase -> 'marker data dto'."""
    return ' '.join(w.lower() for w in split_pascal(name))

def pascal_to_verb(name):
    """Convert PascalCase verb to third-person: 'ApplyUpdate' -> 'Applies an update'."""
    words = split_pascal(name)
    if not words:
        return name.lower()
    verb = words[0].lower()
    if verb.endswith('s') or verb.endswith('sh') or verb.endswith('ch') or verb.endswith('x') or verb.endswith('z'):
        verb += 'es'
    elif verb.endswith('y') and len(verb) > 1 and verb[-2] not in 'aeiou':
        verb = verb[:-1] + 'ies'
    else:
        verb += 's'
    rest = ' '.join(w.lower() for w in words[1:])
    if rest:
        return f"{verb} {rest}"
    return verb

def gen_fn_doc(name, lines_before_fn, has_module_doc):
    """Generate a proper doc comment for a function."""
    if not name:
        return None
    # For forwarding functions named 'Fn' that have module-level doc, skip
    if name == 'Fn' and has_module_doc:
        return None
    if name == 'Fn':
        return None  # too generic
    
    # Snake_case function names: convert underscores to spaces
    if '_' in name:
        desc = name.replace('_', ' ').capitalize()
        return f'/// {desc}.'
    
    # PascalCase function names
    words = split_pascal(name)
    if len(words) >= 2:
        # It's a verb phrase like "ConnectToSideCar"
        desc = pascal_to_verb(name)
        return f'/// {desc}.'
    elif len(words) == 1:
        # Single Pascal word like "IsRunning"
        desc = name.lower()
        return f'/// {desc.capitalize()}.'
    
    return f'/// {name}.'

def gen_item_doc(item_type, name, has_module_doc):
    """Generate doc comment for a pub item."""
    if not name:
        return None
    if item_type == 'fn':
        return gen_fn_doc(name, [], has_module_doc)
    if item_type == 'struct':
        desc = pascal_to_desc(name)
        return f'/// {desc.capitalize()}.'
    if item_type == 'enum':
        desc = pascal_to_desc(name)
        return f'/// Enumeration of {desc}.'
    if item_type == 'trait':
        desc = pascal_to_desc(name)
        return f'/// Trait for {desc}.'
    if item_type == 'type':
        desc = name.replace('_', ' ').lower()
        return f'/// Type alias for {desc}.'
    if item_type in ('const', 'static'):
        desc = name.replace('_', ' ').lower()
        return f'/// {desc.capitalize()}.'
    if item_type == 'mod':
        desc = pascal_to_desc(name)
        return f'/// {desc.capitalize()} module.'
    if item_type == 'macro':
        return '/// Macro definition.'
    return None

# ─── META PATTERN REMOVAL ──────────────────────────

META_PATTERNS = [
    (re.compile(r'^/// This function (does|builds|creates|returns|sends|receives|handles|processes|manages|validates|checks|resolves|loads|parses|queries|updates|deletes|starts|stops|provides|performs|sets|gets|runs|executes|generates|extracts|merges|inserts|removes|closes|opens|writes|reads|configures|finds|collects|enables|disables|registers|unregisters|refreshes|restores|persists|dispatches|cancels|tracks|monitors|launches|spawns|cleans|sweeps|seeds) '),
     lambda m: f'/// {m.group(1).capitalize()} '),
    (re.compile(r'^/// This is (?:a |an |the )?'), lambda m: '/// '),
    (re.compile(r'^/// Represents '), lambda m: '/// '),
    (re.compile(r'^/// This (struct|enum|trait|module|type|macro|const|static|method|class) '), lambda m: '/// '),
    (re.compile(r'^/// This (?:function|module) is responsible for '), lambda m: '/// '),
]

def fix_meta(line):
    """Fix meta-instructional text in doc comments."""
    for pat, repl in META_PATTERNS:
        if pat.search(line):
            return pat.sub(repl, line)
    return line

# ─── PUB ITEM DETECTION ──────────────────────────

def detect_item_type_name(stripped):
    m = re.match(r'pub\s+(struct|enum|trait|type|mod)\s+([a-zA-Z_][a-zA-Z0-9_]*)', stripped)
    if m: return m.group(1), m.group(2)
    m = re.match(r'pub\s+const\s+([a-zA-Z_][a-zA-Z0-9_]*)', stripped)
    if m: return 'const', m.group(1)
    m = re.match(r'pub\s+static\s+([a-zA-Z_][a-zA-Z0-9_]*)', stripped)
    if m: return 'static', m.group(1)
    m = re.match(r'pub\s+(?:async\s+)?(?:unsafe\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)', stripped)
    if m: return 'fn', m.group(1)
    m = re.match(r'pub\s+macro_rules!\s*([a-zA-Z_][a-zA-Z0-9_]*)', stripped)
    if m: return 'macro', m.group(1)
    return None, None

def has_module_doc_before(lines, item_line):
    """Check if there's a //! module-level doc block before this item line."""
    for i in range(min(item_line, 10)):
        if lines[i].strip().startswith("//!"):
            return True
    return False

# ─── FILE PROCESSING ──────────────────────────────

def process_file(filepath):
    with open(filepath, 'r', encoding='utf-8') as f:
        original = f.read()
    lines = original.splitlines(keepends=True)

    # Stage 1: Fix doc comment text
    for i in range(len(lines)):
        s = lines[i].strip()
        if s.startswith("///") and not s.startswith("////") and len(s) > 3:
            fixed = fix_meta(lines[i])
            if fixed != lines[i]:
                lines[i] = fixed

    # Stage 2: Remove blank doc lines
    lines = [l for l in lines if l.strip() != "///"]

    # Stage 3: Collapse consecutive empty lines
    collapsed = []
    prev_empty = False
    for l in lines:
        empty = l.strip() == ""
        if empty and prev_empty:
            continue
        collapsed.append(l)
        prev_empty = empty
    lines = collapsed

    # Stage 4: Add missing docs on pub items
    pub_items = []
    for i, line in enumerate(lines):
        s = line.strip()
        if not s.startswith("pub "):
            continue
        # Collect preceding /// lines
        doc_lines = []
        j = i - 1
        while j >= 0:
            prev = lines[j].strip()
            if prev.startswith("///"):
                doc_lines.insert(0, prev)
            elif prev.startswith("#[") or prev.startswith("#!["):
                pass
            elif prev == "":
                break
            else:
                break
            j -= 1
        item_type, item_name = detect_item_type_name(s)
        if item_type:
            pub_items.append({
                'line': i, 'type': item_type, 'name': item_name,
                'has_doc': len(doc_lines) > 0,
            })

    md = has_module_doc_before(lines, 0)  # has module docs at top of file
    for item in reversed(pub_items):
        if not item['has_doc']:
            doc = gen_item_doc(item['type'], item['name'], md)
            if doc:
                lines.insert(item['line'], doc + '\n')

    content = ''.join(lines)
    if content != original:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)
        return True
    return False

def should_process(filepath):
    rel = os.path.relpath(filepath, SOURCE_DIR)
    if os.path.basename(filepath) == "Library.rs":
        return True
    return any(rel.startswith(m) or rel == f"{m}.rs" for m in TARGET_MODULES)

def is_generated(filepath):
    name = os.path.basename(filepath)
    return name in ("vine.rs",) or "/Generated/" in filepath

def main():
    all_files = []
    for root, _, files in os.walk(SOURCE_DIR):
        for f in files:
            if f.endswith('.rs'):
                fp = os.path.join(root, f)
                if should_process(fp):
                    all_files.append(fp)
    lib = os.path.join(SOURCE_DIR, "Library.rs")
    if os.path.exists(lib) and lib not in all_files:
        all_files.append(lib)
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
            print(f"  ✓ {os.path.relpath(fp, SOURCE_DIR)}")
    total = len(all_files)
    print(f"\nDone: {changed} modified, {total - changed - skipped} unchanged, {skipped} generated skipped")

if __name__ == "__main__":
    main()
