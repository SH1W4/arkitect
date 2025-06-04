"""
ARK_Σ :: engine_interface.py
Módulo integrador do ARK_Σ_Module, coordenando expansão, organização e governança dos conceitos.
"""

from arkitect_engine.ark_expander import ArkExpander
from arkitect_engine.cluster_orchestrator import ClusterOrchestrator
from arkitect_engine.autogov_reflector import AutogovReflector


class ARKSemanticEngine:
    def __init__(self):
        self.expander = ArkExpander()
        self.cluster = ClusterOrchestrator()
        self.reflector = AutogovReflector()

    def process(self, seed: str) -> dict:
        """
        Executa o ciclo completo do ARK_Σ a partir de um conceito-semente.
        """
        expanded = self.expander.expand(seed)
        clusters = self.cluster.organize(expanded)
        insights = {}

        for theme, group in clusters.items():
            insights[theme] = self.reflector.reflect(group)

        return {
            "expanded": expanded,
            "clusters": clusters,
            "reflections": insights
        }
