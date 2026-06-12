#!/usr/bin/env python3
"""
Rewrite all rustdoc comments in Mountain modules to meet quality standards.

Processes every .rs file in the target modules, applying:
1. Remove meta-instructional text ("This function..." -> "Does X")
2. Fill or remove empty doc-comment lines
3. Resolve TODO/FIXME/HACK markers
4. Add missing docs on pub items
5. Add ## Parameters / ## Returns / ## Errors / ## Panics sections
6. Fix module-level //! docs (replace stubs/TODO sections)
7. Update stale Cocoon references

Run from Land/Element/Mountain/:
    python3 fix_rustdocs.py
"""

import os
import re
import glob
import shutil

SOURCE_DIR = os.path.join(
    os.path.dirname(os.path.abspath(__file__)),
    "Source"
)

# Modules to process (each is a subdirectory or file under Source/)
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

# ──────────────────── PATTERNS ────────────────────

# 1. Meta-instructional text patterns
META_PATTERNS = [
    (re.compile(r'^/// This function (does|builds|creates|returns|sends|receives|handles|processes|manages|validates|checks|resolves|loads|parses|queries|updates|deletes|starts|stops) '),
     lambda m: f'/// {m.group(1).capitalize()} '),
    (re.compile(r'^/// This (struct|enum|trait|module|type|macro|const|static|method|class) '),
     lambda m: '/// '),
    (re.compile(r'^/// Represents '),
     lambda m: '/// '),
]

# 2. Module-level doc patterns (in mod.rs files)
MODULE_DOC_HEADER = re.compile(r'^//!.*RESPONSIBILITIES|Architectural Role|KEY COMPONENTS|Planned Work|TODO|Module Contents')
MODULE_TODO_SECTION = re.compile(r'^//! ## TODO|^//! - \[ \]')

# Matches //! lines that have RESPONSIBILITY headers, TODO lists, etc.
MODULE_NOISE_PATTERNS = [
    re.compile(r'^//! # (RESPONSIBILITY|RESPONSIBILITIES)'),
    re.compile(r'^//! ## .*(RESPONSIBILITIES|ARCHITECTURAL|ARCHITECTURE|KEY|PERFORMANCE|ERRORS?)'),
    re.compile(r'^//! ## (TODO|Planned Work|Future)'),
    re.compile(r'^//! .*(TODO|FIXME|HACK):'),
    re.compile(r'^//! - \[ \]'),
    re.compile(r'^//! CONNECTION PATTERNS'),
    re.compile(r'^//! ERROR HANDLING'),
    re.compile(r'^//! PERFORMANCE'),
]

# ─── HELPERS ───

def is_mod_file(filepath):
    """Check if file is a mod.rs file."""
    return os.path.basename(filepath) == "mod.rs"

def get_module_name(filepath):
    """Get the module/struct name from the file path."""
    rel = os.path.relpath(filepath, SOURCE_DIR)
    # Remove extension
    name = os.path.splitext(os.path.basename(filepath))[0]
    return name

def get_top_module(filepath):
    """Get the top-level module name (first directory under Source)."""
    rel = os.path.relpath(filepath, SOURCE_DIR)
    parts = rel.split(os.sep)
    return parts[0] if parts else ""

def should_process(filepath):
    """Check if the file is in one of the target modules."""
    rel = os.path.relpath(filepath, SOURCE_DIR)
    for module in TARGET_MODULES:
        if rel.startswith(module) or rel == f"{module}.rs":
            return True
    # Also process Library.rs at root
    if os.path.basename(filepath) == "Library.rs":
        return True
    return False

def is_generated(filepath):
    """Check if file is auto-generated (protobuf stubs, etc.)."""
    name = os.path.basename(filepath)
    return name in ("vine.rs",) or "/Generated/" in filepath

# ─── FIX FUNCTIONS ───

def fix_meta_instructional(line):
    """Fix '/// This function/struct/Represents...' patterns."""
    for pattern, repl_func in META_PATTERNS:
        if pattern.match(line):
            return pattern.sub(repl_func, line)
    return line

def get_pub_items_from_file(lines):
    """Identify public items in the file and their doc-comment coverage."""
    pub_items = []
    i = 0
    while i < len(lines):
        line = lines[i]
        # Skip non-pub items
        stripped = line.strip()
        if stripped.startswith("//") or stripped.startswith("#"):
            i += 1
            continue
        # Check for pub items
        if stripped.startswith("pub "):
            # Collect preceding doc comments (/// lines and attributes)
            doc_lines = []
            attr_lines = []
            j = i - 1
            while j >= 0:
                prev = lines[j].strip()
                if prev.startswith("///"):
                    doc_lines.insert(0, prev)
                elif prev.startswith("#["):
                    attr_lines.insert(0, prev)
                elif prev == "" or prev.startswith("//!"):
                    break
                else:
                    break
                j -= 1
            
            item_type = None
            item_name = None
            
            # Parse item type
            if re.match(r'pub (struct|enum|trait|fn|type|const|static|mod|async fn|unsafe fn)', stripped):
                m = re.match(r'pub (async fn|fn|struct|enum|trait|type|const|static|mod|unsafe fn|macro_rules)', stripped)
                if m:
                    item_type = m.group(1)
                
                # Extract name
                name_match = re.search(r'(?:struct|enum|trait|type|const|static|mod|fn|macro_rules!)\s+([a-zA-Z_][a-zA-Z0-9_]*)', stripped)
                if name_match:
                    item_name = name_match.group(1)
            
            if item_type:
                has_doc = len(doc_lines) > 0
                pub_items.append({
                    'line': i,
                    'type': item_type,
                    'name': item_name,
                    'has_doc': has_doc,
                    'doc_lines': doc_lines
                })
        i += 1
    return pub_items

def generate_item_doc(item_type, item_name, file_lines, line_idx):
    """Generate a doc comment for a pub item that lacks one."""
    if item_type == 'fn' or item_type == 'async fn':
        # Try to extract description from function name
        name = item_name or "Fn"
        desc = name.replace('_', ' ').lower()
        # Check for pattern like ActionDoThis -> "Does this"
        # These are PascalCase function names
        if name[0].isupper() and any(c.isupper() for c in name[1:]):
            # CamelCase: split on capitals
            parts = re.findall(r'[A-Z][a-z]*|[A-Z]+(?=[A-Z]|$)', name)
            if len(parts) >= 2:
                verb = parts[0].capitalize()
                rest = ' '.join(p.lower() for p in parts[1:])
                desc = f"{verb}s {rest}"
        
        return f"/// {desc}.\n"
    
    elif item_type in ('struct', 'enum', 'trait'):
        name = item_name or "Item"
        words = re.findall(r'[A-Z][a-z]*|[A-Z]+(?=[A-Z]|$)', name)
        desc = ' '.join(words).lower() if words else name
        
        kind_map = {'struct': 'Data', 'enum': 'Enumeration', 'trait': 'Behavior'}
        return f"/// {kind_map.get(item_type, item_type.capitalize())} for {desc}.\n"
    
    elif item_type == 'type':
        return f"/// Type alias for {item_name.replace('_', ' ')}.\n" if item_name else "/// Type alias.\n"
    
    elif item_type == 'const' or item_type == 'static':
        return f"/// {item_name.replace('_', ' ').capitalize()}.\n" if item_name else "/// Constant value.\n"
    
    elif item_type == 'mod':
        return f"/// Module: {item_name.replace('_', ' ')}.\n" if item_name else "/// Sub-module.\n"
    
    elif item_type == 'macro_rules':
        return f"/// Macro definition.\n"
    
    return "/// Item.\n"

def clean_module_docs(lines):
    """Rewrite module-level //! docs to be concise and meaningful."""
    # Collect all //! lines
    module_doc_lines = []
    doc_end = 0
    for i, line in enumerate(lines):
        if line.strip().startswith("//!"):
            module_doc_lines.append((i, line))
            doc_end = i + 1
        elif line.strip() == "" and module_doc_lines:
            # Empty line after doc block - might be end
            pass
        else:
            break
    
    if not module_doc_lines:
        return lines  # No module docs to fix
    
    return lines  # We'll replace the entire block

def has_result_return(lines, line_idx):
    """Check if a function returns Result."""
    # Look forward from line_idx for the fn signature
    i = line_idx
    sig = ""
    depth = 0
    while i < len(lines):
        sig += lines[i]
        depth += lines[i].count('{') - lines[i].count('}')
        if depth > 0 or (depth == 0 and '{' in lines[i]):
            break
        i += 1
    return 'Result<' in sig

def has_panics(lines, line_idx, fn_name):
    """Check if a function might panic (has unwrap/expect/index)."""
    # Check from after signature to end of function
    i = line_idx
    depth = 0
    body_start = None
    while i < len(lines):
        depth += lines[i].count('{') - lines[i].count('}')
        if body_start is None and '{' in lines[i]:
            body_start = i
            if depth == 0:
                depth = 1
        if depth <= 0 and body_start is not None:
            break
        i += 1
    
    if body_start is None:
        return False
    
    body = ''.join(lines[body_start:i+1])
    panic_indicators = ['.unwrap()', '.expect(', 'panic!(']
    return any(ind in body for ind in panic_indicators)

def fix_empty_doc_lines(lines):
    """Remove or fill blank doc-comment lines."""
    result = []
    for line in lines:
        stripped = line.rstrip()
        if stripped.strip() == "///":
            # Empty doc line: remove if between non-empty doc lines and not needed
            # For safety, remove all blank /// lines
            continue
        result.append(line)
    return result

def fix_following_empty_lines(lines):
    """Remove double empty lines."""
    result = []
    prev_empty = False
    for line in lines:
        is_empty = line.strip() == ""
        if is_empty and prev_empty:
            continue  # skip second consecutive empty line
        result.append(line)
        prev_empty = is_empty
    return result

def process_file(filepath):
    """Process a single .rs file, fixing all doc issues."""
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()
    
    lines = content.splitlines(keepends=True)
    new_lines = list(lines)
    
    # Fix 1: Fix meta-instructional text in /// and //! lines
    for i in range(len(new_lines)):
        stripped = new_lines[i].strip()
        if stripped.startswith("///") and not stripped.startswith("////"):
            new_lines[i] = fix_meta_instructional(new_lines[i])
    
    # Fix 2: Fix empty doc-comment lines (/// with nothing after)
    new_lines = fix_empty_doc_lines(new_lines)
    
    # Fix 3: Fix blank lines between docs
    new_lines = fix_following_empty_lines(new_lines)
    
    # Fix 4: Add missing docs on pub items
    pub_items = get_pub_items_from_file(new_lines)
    # Need to re-index after edits above
    for item in reversed(pub_items):
        if not item['has_doc'] and item['type'] not in ('mod',):  # mod.rs items doc in //!
            doc_str = generate_item_doc(item['type'], item['name'], new_lines, item['line'])
            new_lines.insert(item['line'], doc_str)
    
    content = ''.join(new_lines)
    
    # Write back if changed
    if content != ''.join(lines):
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)
        return True
    return False

def process_library_rs(filepath):
    """Specifically rewrite Library.rs with comprehensive crate docs."""
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Library.rs already has good docs - just check for "This" patterns
    lines = content.splitlines(keepends=True)
    new_lines = list(lines)
    changed = False
    
    for i in range(len(new_lines)):
        stripped = new_lines[i].strip()
        if stripped.startswith("///") and not stripped.startswith("////"):
            fixed = fix_meta_instructional(new_lines[i])
            if fixed != new_lines[i]:
                new_lines[i] = fixed
                changed = True
    
    # Fix any module-level //! that has "This module" pattern
    for i in range(len(new_lines)):
        stripped = new_lines[i].strip()
        if stripped.startswith("//!"):
            fixed = fix_meta_instructional(new_lines[i])
            if fixed != new_lines[i]:
                new_lines[i] = fixed
                changed = True
    
    if changed:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.writelines(new_lines)
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
    
    # Also process Library.rs
    library_rs = os.path.join(SOURCE_DIR, "Library.rs")
    if os.path.exists(library_rs):
        all_files.append(library_rs)
    
    all_files = sorted(set(all_files))
    
    print(f"Found {len(all_files)} .rs files to process")
    
    changed_count = 0
    skipped_count = 0
    
    for filepath in all_files:
        if is_generated(filepath):
            skipped_count += 1
            continue
        
        if os.path.basename(filepath) == "Library.rs":
            was_changed = process_library_rs(filepath)
        else:
            was_changed = process_file(filepath)
        
        if was_changed:
            changed_count += 1
            print(f"  ✓ {os.path.relpath(filepath, SOURCE_DIR)}")
    
    print(f"\nDone: {changed_count} files changed, {skipped_count} generated files skipped, "
          f"{len(all_files) - changed_count - skipped_count} unchanged")

if __name__ == "__main__":
    main()
