# Módulo para visualização de estrutura de diretórios
import os

def get_structure(path):
    tree = []
    for root, dirs, files in os.walk(path):
        level = root.replace(path, '').count(os.sep)
        indent = ' ' * 4 * level
        tree.append(f'{indent}{os.path.basename(root)}/')
        subindent = ' ' * 4 * (level + 1)
        for f in files:
            tree.append(f'{subindent}{f}')
    return '\n'.join(tree)