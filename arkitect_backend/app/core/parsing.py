import zipfile
import os

def extract_and_parse_zip(zip_path: str) -> dict:
    extract_dir = zip_path.replace(".zip", "")
    with zipfile.ZipFile(zip_path, 'r') as zip_ref:
        zip_ref.extractall(extract_dir)
    return build_tree_dict(extract_dir)

def build_tree_dict(path: str) -> dict:
    """Gera uma árvore de diretórios como dicionário."""
    name = os.path.basename(path)
    if os.path.isdir(path):
        return {
            "type": "directory",
            "name": name,
            "children": [build_tree_dict(os.path.join(path, x)) for x in os.listdir(path)]
        }
    else:
        return {"type": "file", "name": name}
