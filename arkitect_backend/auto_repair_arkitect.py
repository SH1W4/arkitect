# auto_repair_arkitect.py
import os

REQUIRED_PATHS = {
    "app/__init__.py": "",
    "app/api/__init__.py": "",
    "app/arkitect_engine/__init__.py": "",
    "app/arkitect_engine/ark_expander.py": '''\
class ArkExpander:
    def expand(self, seed: str):
        return [seed, f"{seed}_exp1", f"{seed}_exp2"]
'''
}

def ensure_structure(base_dir):
    created = []
    for rel_path, content in REQUIRED_PATHS.items():
        full_path = os.path.join(base_dir, rel_path)
        dir_path = os.path.dirname(full_path)

        if not os.path.exists(dir_path):
            os.makedirs(dir_path)
            created.append(f"[+] Criado diretório: {dir_path}")

        if not os.path.isfile(full_path):
            with open(full_path, "w", encoding="utf-8") as f:
                f.write(content)
            created.append(f"[+] Criado arquivo: {rel_path}")
        else:
            created.append(f"[=] Arquivo existente preservado: {rel_path}")
    return created

if __name__ == "__main__":
    BASE_DIR = os.path.dirname(os.path.abspath(__file__))
    results = ensure_structure(BASE_DIR + "/app")
    
    print("\n=== ARK_Σ Auto Repair Log ===")
    for line in results:
        print(line)
    print("✅ Estrutura verificada e corrigida com sucesso.")
