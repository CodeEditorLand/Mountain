#!/usr/bin/env python3
"""
Final cleanup for IPC docs: remove orphaned boilerplate content that lost its section headers.
"""
import re
from pathlib import Path

IPC_DIR = Path("/Volumes/CORSAIR/Developer/macOS/Application/CodeEditorLand/Land/Element/Mountain/Source/IPC")

# Patterns of orphaned boilerplate content
ORPHAN_LINES = [
    # **External Crates:** / **Internal Modules:** blocks
    re.compile(r'^//!\s*\*\*External Crates:\*\*'),
    re.compile(r'^//!\s*\*\*Internal Modules:\*\*'),
    # Standalone bullet lists that reference dependencies or implementors
    re.compile(r'^//!\s*-\s*`(TauriIPCServer|RouteMessage|Compress|Encrypt|Send|Receive)`'),
    # Orphaned continuation of dependency sections
    re.compile(r'^//!\s*-\s*None\s*\(this\s+module\s+provides'),
]

# Module-level docs that should have ## Overview instead of no section header
GOOD_FIRST_LINE = re.compile(r'^//!\s*#\s+\w+')

def process_file(filepath):
    with open(filepath, 'r', encoding='utf-8', errors='replace') as f:
        original = f.read()
    
    lines = original.split('\n')
    modified = False
    
    # Remove orphaned lines
    new_lines = []
    i = 0
    while i < len(lines):
        line = lines[i]
        stripped = line.strip()
        is_mod = stripped.startswith('//!') and not stripped.startswith('//!!')
        
        if is_mod:
            orphan = False
            for pat in ORPHAN_LINES:
                if pat.match(stripped):
                    # Remove this line and any following mod-lines that are continuation bullets
                    orphan = True
                    modified = True
                    break
            if orphan:
                # Skip continuation lines (indented bullets or blank mod lines)
                i += 1
                while i < len(lines):
                    next_stripped = lines[i].strip()
                    if next_stripped.startswith('//!') and not next_stripped.startswith('//!!'):
                        content = next_stripped.lstrip('/!> \t')
                        if content.startswith('-') or content.startswith('*') or content.startswith('`') or not content.strip():
                            modified = True
                            i += 1
                            continue
                    break
                continue
        
        new_lines.append(line)
        i += 1
    
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
    mc = 0
    for idx, fp in enumerate(rs_files):
        try:
            if process_file(fp):
                mc += 1
                print(f"  [{idx+1}/{total}] CLEANED: {fp.relative_to(IPC_DIR)}")
        except Exception as e:
            print(f"  [{idx+1}/{total}] ERROR: {fp.relative_to(IPC_DIR)}: {e}")
    print(f"\nDone: {mc} cleaned, {total} total")

if __name__ == '__main__':
    main()
