# Lógica para lidar com uploads .zip/.rar
import zipfile
import os

def extract_zip(zip_file, extract_to='uploaded_project'):
    with zipfile.ZipFile(zip_file, 'r') as zip_ref:
        zip_ref.extractall(extract_to)
    return extract_to