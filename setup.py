from setuptools import setup, find_packages

setup(
    name="arkitect_backend",
    version="1.0.0",
    packages=find_packages(),
    install_requires=[
        "fastapi>=0.70.0",
        "uvicorn>=0.15.0",
        "redis>=4.0.0",
        "python-dotenv>=0.19.0",
        "loguru>=0.5.3",
        "pydantic>=1.8.2",
        "requests>=2.26.0",
        "pytest>=6.2.5",
        "python-jose[cryptography]>=3.3.0",
        "python-multipart>=0.0.5",
    ],
    extras_require={
        "dev": [
            "black>=21.9b0",
            "flake8>=3.9.2",
            "mypy>=0.910",
            "pytest>=6.2.5",
        ]
    },
    python_requires=">=3.9",
)

