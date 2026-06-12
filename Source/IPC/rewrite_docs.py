#!/usr/bin/env python3
"""
Rewrite all rustdoc comments in Mountain's IPC module to meet quality standards.
"""
import os, re, sys
from pathlib import Path

IPC_DIR = Path("/Volumes/CORSAIR/Developer/macOS/Application/CodeEditorLand/Land/Element/Mountain/Source/IPC")

THIS_VERB_RE   = re.compile(r'^(///)\s*This\s+(struct|function|enum|trait|module|type|method|macro|const|static|structure)\s+(.+)$', re.I)
THIS_VERB_MOD_RE = re.compile(r'^(//!)\s*This\s+(struct|function|enum|trait|module|type|method|macro|const|static|structure)\s+(.+)$', re.I)
REPRESENTS_RE  = re.compile(r'^(///)\s*Represents\s+(.+)$', re.I)
REPRESENTS_MOD_RE = re.compile(r'^(//!)\s*Represents\s+(.+)$', re.I)
TODO_START_RE  = re.compile(r'^(//[!>])\s*#+\s*TODO(?:\s+Items?)?\s*$', re.I)

def fix_this_verb(line, pattern):
    m = pattern.match(line)
    if not m: return line
    prefix, rest = m.group(1), m.group(3)
    if rest and rest[0].islower(): rest = rest[0].upper() + rest[1:]
    return f"{prefix} {rest}"

def fix_represents(line, pattern):
    m = pattern.match(line)
    if not m: return line
    prefix, content = m.group(1), m.group(2)
    if content and content[0].islower(): content = content[0].upper() + content[1:]
    return f"{prefix} {content}"

# Sections to strip entirely (header line + all following content until next section or empty line stops it)
STRIP_SECTION_HEADERS = [
    'File: IPC/', 'Role in Mountain Architecture', 'Role:', 
    'Primary Responsibility', 'Secondary Responsibilities',
    'Dependencies', 'Dependents',
    'VSCode Pattern Reference', 'Security Consideration',
    'Thread Safety', 'Error Handling Strategy',
]

BOILERPLATE_SECTION = re.compile(
    r'^//!\s*##\s+(' + '|'.join(re.escape(h) for h in STRIP_SECTION_HEADERS) + r')', re.I)

# Lines like "N/A - This is a data definition module."
BOILERPLATE_NOP = re.compile(
    r'^//!\s*(N/A\s*[-–]|This\s+is\s+a\s+data\s+definition)', re.I)

def is_todo_comment(stripped):
    """Check if a line belongs to a // ## TODO section"""
    if TODO_START_RE.match(stripped): return True
    s = stripped.lstrip('/!> \t')
    if not s.strip(): return True
    return bool(re.match(r'^[-*\[]\s', s.strip()))

def process_file(filepath):
    with open(filepath, 'r', encoding='utf-8', errors='replace') as f:
        original = f.read()
    
    lines = original.split('\n')
    modified = False
    in_todo = False
    skip_mod_block = False  # Skip a //! block that is boilerplate (section + its content)
    skip_mod_count = 0
    
    result = []
    i = 0
    while i < len(lines):
        line = lines[i]
        stripped = line.strip()
        is_doc = stripped.startswith('///') and not stripped.startswith('////')
        is_mod = stripped.startswith('//!') and not stripped.startswith('//!!')
        is_plain_comment = stripped.startswith('//') and not is_doc and not is_mod and stripped != '//'
        
        # ---------- Track // ## TODO blocks ----------
        if is_plain_comment and not in_todo and TODO_START_RE.match(stripped):
            in_todo = True
            modified = True
            i += 1
            continue
        if in_todo:
            if is_plain_comment and is_todo_comment(stripped):
                modified = True
                i += 1
                continue
            else:
                in_todo = False  # fall through to process this line
        
        # ---------- Track //! ## TODO blocks ----------
        if is_mod and not in_todo and TODO_START_RE.match(stripped):
            in_todo = True
            modified = True
            i += 1
            continue
        if in_todo and is_mod:
            s = stripped.lstrip('/!> \t')
            if not s.strip() or re.match(r'^[-*\[]\s', s.strip()):
                modified = True
                i += 1
                continue
            else:
                in_todo = False
        in_todo = False
        
        # ---------- Strip boilerplate sections (header + content) ----------
        if is_mod and BOILERPLATE_SECTION.match(stripped):
            modified = True
            skip_mod_block = True
            skip_mod_count = 0
            i += 1
            continue
        
        if skip_mod_block:
            if is_mod:
                # Continue skipping content lines of the block
                s2 = stripped.lstrip('/!> \t')
                # Keep going if it's a continuation (starts with ** or - or space or is empty)
                if (not s2.strip() or s2.startswith('**') or s2.startswith('- ') or 
                    s2.startswith('* ') or s2[0:1].isupper() and ' ' in s2[:20]):
                    skip_mod_count += 1
                    modified = True
                    i += 1
                    continue
                else:
                    # Not continuation - emit this line
                    skip_mod_block = False
                    result.append(line)
                    i += 1
                    continue
            else:
                skip_mod_block = False
                result.append(line)
                i += 1
                continue
        
        # ---------- N/A boilerplate ----------
        if is_mod and BOILERPLATE_NOP.match(stripped):
            modified = True
            i += 1
            continue
        
        # ---------- Fix This Struct/Function ... ----------
        if is_mod and THIS_VERB_MOD_RE.match(stripped):
            nl = fix_this_verb(line, THIS_VERB_MOD_RE)
            if nl != line: modified = True
            result.append(nl)
            i += 1
            continue
        
        if is_mod and REPRESENTS_MOD_RE.match(stripped):
            nl = fix_represents(line, REPRESENTS_MOD_RE)
            if nl != line: modified = True
            result.append(nl)
            i += 1
            continue
        
        # ---------- Fix /// items ----------
        if is_doc and THIS_VERB_RE.match(stripped):
            nl = fix_this_verb(line, THIS_VERB_RE)
            if nl != line: modified = True
            result.append(nl)
            i += 1
            continue
        
        if is_doc and REPRESENTS_RE.match(stripped):
            nl = fix_represents(line, REPRESENTS_RE)
            if nl != line: modified = True
            result.append(nl)
            i += 1
            continue
        
        result.append(line)
        i += 1
    
    # Collapse consecutive empty doc/mod lines
    collapsed = []
    prev_empty = False
    for line in result:
        s = line.strip()
        is_empty = s in ('///', '/// ', '//!', '//! ')
        if is_empty:
            if prev_empty:
                modified = True
                continue
            prev_empty = True
        else:
            prev_empty = False
        collapsed.append(line)
    
    new_content = '\n'.join(collapsed)
    if new_content != original:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(new_content)
        return True
    return False

def main():
    rs_files = sorted(IPC_DIR.rglob('*.rs'))
    rs_files = [f for f in rs_files if f.name != 'rewrite_docs.py']
    total = len(rs_files)
    mc = ec = 0
    print(f"Found {total} .rs files in {IPC_DIR}")
    for idx, fp in enumerate(rs_files):
        try:
            if process_file(fp):
                mc += 1
                print(f"  [{idx+1}/{total}] MODIFIED: {fp.relative_to(IPC_DIR)}")
        except Exception as e:
            ec += 1
            print(f"  [{idx+1}/{total}] ERROR: {fp.relative_to(IPC_DIR)}: {e}", file=sys.stderr)
        if (idx+1) % 100 == 0:
            print(f"  Progress: {idx+1}/{total}, {mc} modified, {ec} errors")
    print(f"\nDone: {mc} modified, {ec} errors, {total} total")

if __name__ == '__main__':
    main()
