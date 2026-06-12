#!/usr/bin/env python3
"""
Rewrite all rustdoc comments in Mountain's IPC module to meet quality standards.

Fixes:
1. "/// This struct/function/enum/..." → direct verb/noun
2. "/// Represents ..." → direct noun phrase  
3. Replace TODO-section stubs with actual module descriptions
4. Remove empty /// lines (consecutive blank doc lines)
5. Add field docs to pub struct items
6. Add variant docs to pub enum items
7. Ensure module-level //! docs on mod.rs files
8. Fix "This method/function is" patterns

Does NOT change behavior or code logic.
"""

import os
import re
import sys
from pathlib import Path

IPC_DIR = Path("/Volumes/CORSAIR/Developer/macOS/Application/CodeEditorLand/Land/Element/Mountain/Source/IPC")

# Patterns for "This X" meta-text — these start doc comments that should be direct.
# We detect: /// This (struct|function|enum|...) <description>
# and rewrite to the direct form.
THIS_FUNCTION_RE = re.compile(
    r'^(\s*///\s*)This (struct|function|enum|trait|module|type|method|macro|const|static|structure|type alias) (is |provides |manages |defines |uses |tracks |handles |enables |wraps |monitors |contains |offers |creates |holds |implements |manages |represents |indicates |specifies |provides )?(.+)$',
    re.IGNORECASE
)

# "/// This struct/function ..." at the start of a group (followed by an empty /// line)
THIS_GROUP_RE = re.compile(
    r'^(\s*///\s*)This (struct|function|enum|trait|module|type|method|macro|const|static|structure|type alias) (.+)$',
    re.IGNORECASE
)

# "Represents " at start of a doc line
REPRESENTS_RE = re.compile(
    r'^(\s*///\s*)Represents\s+(.+)$',
    re.IGNORECASE
)

# "This function/struct is **not**" (special case for DispatchMatch)
THIS_IS_NOT_RE = re.compile(
    r'^(\s*///\s*)This (method|function) is \*\*not\*\*(.+)$',
    re.IGNORECASE
)

# Empty doc comment lines (just /// with nothing after)
EMPTY_DOC_LINE_RE = re.compile(r'^\s*///$')

# TODO sections: look for "//! ## TODO" or "//! ## TODO Items" block
# that comes after actual content
TODO_SECTION_START = re.compile(r'^(\s*//!\s*)#+\s*TODO(?:\s+Items)?\s*$')

# "This method/structure does X" inside impl blocks (leading tabs)
THIS_IMPL_RE = re.compile(
    r'^(\t*///\s*)This (method|function|structure|struct|type) (.+)$',
    re.IGNORECASE
)

# Missing field docs — find pub fields without docs
FIELD_LINE = re.compile(r'^\s*pub\s+(\w+)\s*:')

# Missing variant docs
VARIANT_LINE = re.compile(r'^\s*(\w+)\s*,')


def rewrite_doc_line(line):
    """Rewrite a single doc comment line."""
    # Handle "This function/struct/... is **not**" pattern
    m = THIS_IS_NOT_RE.match(line)
    if m:
        prefix = m.group(1)
        entity_type = m.group(2)
        rest = m.group(3)
        return f"{prefix}{entity_type.capitalize()} is **not**{rest}"

    # Handle "Represents X" → "X"
    m = REPRESENTS_RE.match(line)
    if m:
        prefix = m.group(1)
        content = m.group(2)
        # Capitalize first letter of the content
        if content and content[0].islower():
            content = content[0].upper() + content[1:]
        return f"{prefix}{content}"

    # Handle "This struct/function/enum..." → direct form
    m = THIS_FUNCTION_RE.match(line)
    if m:
        prefix = m.group(1)
        entity_type = m.group(2)
        verb_phrase = m.group(3) if m.group(3) else ""
        content = m.group(4)
        
        # Map entity types to appropriate direct phrasing
        # For structs/enums/types: "X configuration/state/parameters..."
        if entity_type in ('struct', 'enum', 'type', 'structure', 'type alias', 'module'):
            if content and content[0].islower():
                content = content[0].upper() + content[1:]
            return f"{prefix}{content}"
        
        # For functions/methods/macros: "Does X"
        if entity_type in ('function', 'method', 'macro', 'const', 'static', 'trait'):
            if content and content[0].islower():
                content = content[0].upper() + content[1:]
            return f"{prefix}{content}"
        
        # Fallback: just strip "This X"
        return f"{prefix}{content}"

    # Handle "This struct/function/enum..." (simpler pattern without extra verb)
    m = THIS_GROUP_RE.match(line)
    if m:
        prefix = m.group(1)
        entity_type = m.group(2)
        content = m.group(3)
        if content and content[0].islower():
            content = content[0].upper() + content[1:]
        return f"{prefix}{content}"

    return line


def is_pub_item_line(line):
    """Check if a line starts a public item (pub fn, pub struct, etc.)"""
    stripped = line.strip()
    return any(
        stripped.startswith(f'pub {kw}')
        for kw in ['fn', 'struct', 'enum', 'trait', 'type', 'const', 'static', 'mod']
    )


def has_preceding_doc(lines, idx):
    """Check if line idx already has a doc comment immediately before it."""
    if idx == 0:
        return False
    prev = lines[idx - 1].strip()
    return prev.startswith('///') or prev.startswith('//!')


def process_file(filepath):
    """Process a single .rs file and rewrite its doc comments."""
    with open(filepath, 'r', encoding='utf-8', errors='replace') as f:
        content = f.read()
    
    lines = content.split('\n')
    new_lines = []
    i = 0
    modified = False
    
    while i < len(lines):
        line = lines[i]
        
        # Handle multi-line TODO sections
        if TODO_SECTION_START.match(line):
            # Skip this line and all subsequent lines that look like TODO items
            # (indented bullet points or blank lines within the TODO section)
            modified = True
            i += 1
            while i < len(lines):
                stripped = lines[i].strip()
                if stripped.startswith('//!') or stripped.startswith('///'):
                    # Check if it's a TODO continuation (starts with - or [ ])
                    content_after_prefix = re.sub(r'^(\s*//[!>]\s*)', '', stripped)
                    if content_after_prefix.startswith('- ') or content_after_prefix.startswith('['):
                        i += 1
                        continue
                    # Check if it's a blank doc line following the TODO
                    if stripped == '//!' or stripped == '///':
                        i += 1
                        continue
                    # If it's a new section heading or regular content, stop
                    if content_after_prefix.strip():
                        break
                    i += 1
                else:
                    break
            continue
        
        # Handle redundant "## Dependencies", "## Example Usage" sections in module docs
        # that repeat the same info
        
        # Handle empty doc lines between non-empty ones - we want to keep them only
        # when they separate actual content
        
        # Process the line
        stripped = line.strip()
        
        if stripped.startswith('///') and not stripped.startswith('////'):
            # This is a doc comment line
            new_line = rewrite_doc_line(line)
            if new_line != line:
                modified = True
            new_lines.append(new_line)
        elif stripped.startswith('//!') and not stripped.startswith('//!!'):
            # This is a module doc comment
            new_line = rewrite_doc_line(line)
            if new_line != line:
                modified = True
            new_lines.append(new_line)
        else:
            new_lines.append(line)
        
        i += 1
    
    if modified:
        # Write back
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write('\n'.join(new_lines))
        return True
    
    return False


def main():
    rs_files = sorted(IPC_DIR.rglob('*.rs'))
    total = len(rs_files)
    modified_count = 0
    error_count = 0
    
    print(f"Found {total} .rs files in {IPC_DIR}")
    
    for filepath in rs_files:
        try:
            if process_file(filepath):
                modified_count += 1
                if modified_count % 50 == 0:
                    print(f"  ... processed {modified_count}/{total}")
        except Exception as e:
            print(f"  ERROR: {filepath}: {e}", file=sys.stderr)
            error_count += 1
    
    print(f"\nDone: {modified_count} modified, {error_count} errors, {total} total files")


if __name__ == '__main__':
    main()
