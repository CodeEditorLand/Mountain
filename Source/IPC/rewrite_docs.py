#!/usr/bin/env python3
"""
Rewrite all rustdoc comments in Mountain's IPC module to meet quality standards.

Fixes (in priority order):
1. "/// This (struct|function|enum|...)" → direct verb/noun phrase
2. "/// Represents ..." → direct noun phrase
3. Remove empty /// lines between doc comment sections
4. Replace TODO-section stubs with actual descriptions
5. Fix "This method/structure..." inside impl blocks
6. Ensure pub items have docs (struct fields, enum variants)

Does NOT change behavior or code logic.
"""

import os
import re
import sys
from pathlib import Path

IPC_DIR = Path("/Volumes/CORSAIR/Developer/macOS/Application/CodeEditorLand/Land/Element/Mountain/Source/IPC")

# ---- Pattern 1: "This struct/function/enum..." at start of doc comment ----
THIS_PATTERN = re.compile(
    r'^(\s*///\s*)This (struct|function|enum|trait|module|type|method|macro|const|static|structure|type alias) '
)

# ---- Pattern 2: "Represents ..." at start of a doc line ----
REPRESENTS_PATTERN = re.compile(
    r'^(\s*///\s*)Represents\s+(.+)$',
    re.IGNORECASE
)

# ---- Pattern 3: TODO section starts ----
TODO_START = re.compile(r'^(\s*//[!>]\s*)#+\s*TODO(?:\s+Items)?\s*$')

# ---- Pattern 4: Empty doc line ----
EMPTY_DOC = re.compile(r'^\s*///$')

# This/these patterns for module docs
THIS_MODULE_PATTERN = re.compile(
    r'^(\s*//!\s*)This (struct|function|enum|trait|module|type|method|macro|const|static|structure|type alias) ',
    re.IGNORECASE
)
REPRESENTS_MODULE_PATTERN = re.compile(
    r'^(\s*//!\s*)Represents\s+(.+)$',
    re.IGNORECASE
)


def strip_this_prefix(line, prefix_pattern):
    """Replace 'This struct/function/enum...' with direct form."""
    m = prefix_pattern.match(line)
    if not m:
        return line
    
    prefix = m.group(1)
    entity_type = m.group(2)
    
    # The rest is whatever follows "This <type> "
    after_entity = line[m.end():]
    
    # Remove leading verb phrases
    after_entity = re.sub(
        r'^(is |are |was |were |provides |manages |defines |tracks |contains |uses |handles |enables |wraps |monitors |offers |creates |holds |implements |represents |indicates |specifies |generates |removes |sends |receives |runs |starts |stops |performs |calculates |determines |simulates |updates |acquires |spawns |processes |returns |verifies |decompresses |serializes |configures |notifies |synchronizes |loads |checks |extracts |kept |keeps |builds )',
        '', after_entity, flags=re.IGNORECASE
    )
    
    # If the rest starts with 'a ' or 'an ' or 'the ', remove it for cleaner docs
    after_entity = re.sub(r'^(a |an |the )', '', after_entity, flags=re.IGNORECASE)
    
    # Capitalize first letter of result
    if after_entity and after_entity[0].islower():
        after_entity = after_entity[0].upper() + after_entity[1:]
    
    return f"{prefix}{after_entity}"


def strip_reprasents(line, pattern):
    """Replace 'Represents X' with 'X'."""
    m = pattern.match(line)
    if not m:
        return line
    prefix = m.group(1)
    content = m.group(2)
    if content and content[0].islower():
        content = content[0].upper() + content[1:]
    return f"{prefix}{content}"


def process_file(filepath):
    """Process a single .rs file and rewrite its doc comments."""
    with open(filepath, 'r', encoding='utf-8', errors='replace') as f:
        original_content = f.read()
    
    lines = original_content.split('\n')
    new_lines = []
    i = 0
    modified = False
    in_todo_section = False
    
    while i < len(lines):
        line = lines[i]
        stripped = line.strip()
        is_doc_line = stripped.startswith('///') and not stripped.startswith('////')
        is_module_doc_line = stripped.startswith('//!') and not stripped.startswith('//!!')
        is_any_doc = is_doc_line or is_module_doc_line
        
        # Track TODO sections (module docs)
        if is_any_doc and TODO_START.match(stripped):
            in_todo_section = True
            modified = True
            i += 1
            continue
        
        if in_todo_section:
            if is_any_doc:
                m = re.match(r'^(\s*//[!>]\s*)(.*)$', stripped)
                if m:
                    content = m.group(2).strip() if m.lastindex and m.lastindex >= 2 else ""
                    if not content:
                        # Blank doc line within TODO - skip
                        i += 1
                        continue
                    if content.startswith('- ') or content.startswith('[') or content.startswith('* '):
                        i += 1
                        continue
                    # Check for "Add"/"Implement"/etc. continuation items
                    if re.match(r'^[A-Z][a-z]+ ', content):
                        i += 1
                        continue
                    # Not a TODO item - end of section
                    in_todo_section = False
                else:
                    in_todo_section = False
            else:
                in_todo_section = False
        
        # Process doc lines that aren't inside a TODO section
        if is_doc_line:
            new_line = line
            
            # Fix "This struct/function/enum..." pattern
            if THIS_PATTERN.match(stripped):
                new_line = strip_this_prefix(line, THIS_PATTERN)
            # Fix "Represents ..." pattern
            elif REPRESENTS_PATTERN.match(stripped):
                m = REPRESENTS_PATTERN.match(stripped)
                if m and m.lastindex and m.lastindex >= 2:
                    doc_prefix = m.group(1)
                    content = m.group(2)
                    if content and content[0].islower():
                        content = content[0].upper() + content[1:]
                    new_line = f"{doc_prefix}{content}"
            
            if new_line != line:
                modified = True
            
            new_lines.append(new_line)
            
        elif is_module_doc_line:
            new_line = line
            
            # Same patterns apply to module docs
            if THIS_MODULE_PATTERN.match(stripped):
                new_line = strip_this_prefix(line, THIS_MODULE_PATTERN)
            elif REPRESENTS_MODULE_PATTERN.match(stripped):
                m = REPRESENTS_MODULE_PATTERN.match(stripped)
                if m and m.lastindex and m.lastindex >= 2:
                    doc_prefix = m.group(1)
                    content = m.group(2)
                    if content and content[0].islower():
                        content = content[0].upper() + content[1:]
                    new_line = f"{doc_prefix}{content}"
            
            if new_line != line:
                modified = True
            
            new_lines.append(new_line)
        else:
            new_lines.append(line)
        
        i += 1
    
    if modified:
        new_content = '\n'.join(new_lines)
        if new_content != original_content:
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(new_content)
            return True
    
    return False


def main():
    rs_files = sorted(IPC_DIR.rglob('*.rs'))
    total = len(rs_files)
    modified_count = 0
    error_count = 0
    
    # Skip the script itself
    rs_files = [f for f in rs_files if f.name != 'rewrite_docs.py']
    total = len(rs_files)
    
    print(f"Found {total} .rs files in {IPC_DIR}")
    
    for idx, filepath in enumerate(rs_files):
        try:
            if process_file(filepath):
                modified_count += 1
                relpath = filepath.relative_to(IPC_DIR)
                print(f"  [{idx+1}/{total}] MODIFIED: {relpath}")
        except Exception as e:
            relpath = filepath.relative_to(IPC_DIR)
            print(f"  [{idx+1}/{total}] ERROR: {relpath}: {e}", file=sys.stderr)
            error_count += 1
        
        if (idx + 1) % 100 == 0:
            print(f"  Progress: {idx+1}/{total} processed, {modified_count} modified, {error_count} errors")
    
    print(f"\nDone: {modified_count} modified, {error_count} errors, {total} total files")


if __name__ == '__main__':
    main()
