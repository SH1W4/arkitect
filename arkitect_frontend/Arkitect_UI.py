import streamlit as st
import requests
import os
from pathlib import Path

st.set_page_config(page_title="🧠 ARKITECT", layout="wide")

st.title("🧠 ARKITECT - Sistema de Documentação Inteligente")

st.markdown("Upload de projetos (.zip ou .rar) para extração de estrutura e geração de documentação automatizada com IA.")

uploaded_file = st.file_uploader("📦 Faça upload de um projeto (ZIP ou RAR)", type=["zip", "rar"])

if uploaded_file:
    file_path = Path("uploaded_projects") / uploaded_file.name
    file_path.parent.mkdir(exist_ok=True)
    with open(file_path, "wb") as f:
        f.write(uploaded_file.getbuffer())

    st.success(f"✅ Arquivo '{uploaded_file.name}' salvo com sucesso!")

    if st.button("🚀 Iniciar Extração"):
        with st.spinner("Analisando projeto..."):
            try:
                response = requests.post("http://localhost:8000/process_project/", files={"file": open(file_path, "rb")})
                if response.status_code == 200:
                    result = response.json()
                    st.success("📊 Estrutura analisada com sucesso!")
                    st.json(result)
                else:
                    st.error("Erro durante o processamento. Verifique o backend.")
            except Exception as e:
                st.error(f"Erro de conexão com backend: {e}")
