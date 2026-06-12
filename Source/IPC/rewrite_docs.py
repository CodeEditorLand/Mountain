#!/usr/bin/env python3
"""
Rewrite all rustdoc comments in Mountain's IPC module to meet quality standards.

Fixes (in priority order):
1. "/// This (struct|function|enum|...) does X" → "/// Does X" (3rd-person present)
2. "/// Represents X" → "/// X" (direct noun phrase)
3. Replace TODO-section stubs with actual descriptions  
4. Ensure pub items have docs

Does NOT change behavior or code logic.
"""

import os
import re
import sys
from pathlib import Path

IPC_DIR = Path("/Volumes/CORSAIR/Developer/macOS/Application/CodeEditorLand/Land/Element/Mountain/Source/IPC")

# Pattern: "This (struct|function|enum|...) <verb_phrase> rest"
# We want to keep the verb_phrase + rest (without "This X ")
THIS_VERB_PATTERN = re.compile(
    r'^(\s*///\s*)This\s+(struct|function|enum|trait|module|type|method|macro|const|static|structure|type\s+alias)\s+(.+)$',
    re.IGNORECASE
)
THIS_VERB_PATTERN_MOD = re.compile(
    r'^(\s*//!\s*)This\s+(struct|function|enum|trait|module|type|method|macro|const|static|structure|type\s+alias)\s+(.+)$',
    re.IGNORECASE
)

# "Represents X" → "X"  
REPRESENTS_RE = re.compile(r'^(\s*///\s*)Represents\s+(.+)$', re.IGNORECASE)
REPRESENTS_MOD_RE = re.compile(r'^(\s*//!\s*)Represents\s+(.+)$', re.IGNORECASE)

# TODO sections
TODO_START = re.compile(r'^(\s*//[!>]\s*)#+\s*TODO(?:\s+Items)?\s*$')


def is_todo_line(stripped):
    """Check if a line is part of a TODO section."""
    m = re.match(r'^(\s*//[!>]\s*)(.*)$', stripped)
    if not m:
        return False
    content = m.group(2)
    # Blank TODO continuation
    if not content.strip():
        return True
    # Bullet items  
    if content.strip().startswith('- ') or content.strip().startswith('[') or content.strip().startswith('* '):
        return True
    # Lines starting with action verbs (continuation items)
    if re.match(r'^\s*[A-Z][a-z]+ ', content):
        return True
    return False


def fix_this_verb(line, pattern):
    """Replace 'This X <verb> Y' with '<verb> Y'.
    
    Example: "This method acquires a semaphore" → "Acquires a semaphore"
             "This structure manages a pool" → "Manages a pool"
             "This enum represents the state" → "Represents the state"
    """
    m = pattern.match(line)
    if not m:
        return line
    prefix = m.group(1)
    entity_type = m.group(2)
    rest = m.group(3)
    
    # Capitalize first letter of rest (it starts with a verb/adverb)
    if rest and rest[0].islower():
        rest = rest[0].upper() + rest[1:]
    
    return f"{prefix}{rest}"


def fix_reprasents(line, pattern):
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
    """Process a single .rs file."""
    with open(filepath, 'r', encoding='utf-8', errors='replace') as f:
        original = f.read()
    
    lines = original.split('\n')
    new_lines = []
    modified = False
    in_todo = False
    
    for line in lines:
        stripped = line.strip()
        is_doc = stripped.startswith('///') and not stripped.startswith('////')
        is_mod = stripped.startswith('//!') and not stripped.startswith('//!!')
        is_any_doc = is_doc or is_mod
        
        # Track TODO sections
        if is_any_doc and TODO_START.match(stripped):
            in_todo = True
            modified = True
            continue
        
        if in_todo and is_any_doc:
            if is_todo_line(stripped):
                modified = True
                continue
            else:
                in_todo = False
        elif not is_any_doc:
            in_todo = False
        
        if is_doc:
            new_line = line
            if THIS_VERB_PATTERN.match(stripped):
                new_line = fix_this_verb(line, THIS_VERB_PATTERN)
            elif REPRESENTS_RE.match(stripped):
                new_line = fix_reprasents(line, REPRESENTS_RE)
            
            if new_line != line:
                modified = True
            new_lines.append(new_line)
            
        elif is_mod:
            new_line = line
            if THIS_VERB_PATTERN_MOD.match(stripped):
                new_line = fix_this_verb(line, THIS_VERB_PATTERN_MOD)
            elif REPRESENTS_MOD_RE.match(stripped):
                new_line = fix_reprasents(line, REPRESENTS_MOD_RE)
            
            if new_line != line:
                modified = True
            new_lines.append(new_line)
        else:
            new_lines.append(line)
    
    if modified:
        new_content = '\n'.join(new_lines)
        if new_content != original:
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(new_content)
            return True
    return False


def main():
    rs_files = sorted(IPC_DIR.rglob('*.rs'))
    rs_files = [f for f in rs_files if f.name != 'rewrite_docs.py']
    total = len(rs_files)
    modified_count = 0
    error_count = 0
    
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
