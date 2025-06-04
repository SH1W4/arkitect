"""
ARK_Σ :: semantic_expansion.py
Módulo responsável pela expansão semântica de conceitos-semente em estruturas cognitivas compostas.
"""

from typing import List, Dict
from arkitect_engine.symbolic_map import SymbolicMap

class SemanticExpander:
    def __init__(self):
        self.map = SymbolicMap()

    def expand(self, seed: str) -> Dict[str, List[str]]:
        """
        Recebe uma semente simbólica e retorna uma expansão semântica organizada.
        """
        base_cluster = self.map.generate_cluster(seed)
        enriched_cluster = self._enrich_cluster(base_cluster)
        return enriched_cluster

    def _enrich_cluster(self, cluster: List[str]) -> Dict[str, List[str]]:
        return {
            "seed": cluster[0],
            "primary": cluster[1:4],
            "extended": cluster[4:]
        }
