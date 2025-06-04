"""
ARK_Σ :: symbolic_map.py
Módulo responsável por gerar mapas simbólicos e redes de significados em torno de uma semente.
"""

from typing import List
import random

class SymbolicMap:
    def __init__(self):
        self.semantic_memory = {
            "ark": ["arca", "estrutura", "nave", "refúgio", "evolução", "código", "matriz", "oráculo"],
            "simbiose": ["união", "fusão", "mutualismo", "coexistência", "interface", "inteligência viva"],
            "governança": ["controle", "ordem", "gestão", "ética", "transparência", "código moral"]
        }

    def generate_cluster(self, seed: str) -> List[str]:
        """
        Gera um cluster simbólico a partir de uma semente, com expansão aleatória limitada.
        """
        base = self.semantic_memory.get(seed.lower(), [seed])
        expansion = random.sample(base, min(len(base), 6))
        return [seed] + expansion
