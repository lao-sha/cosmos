"""Embedding Service for text vectorization"""

from typing import List, Optional
import numpy as np
from sentence_transformers import SentenceTransformer
from ..config import get_settings


class EmbeddingService:
    """Service for generating text embeddings"""
    
    _instance: Optional["EmbeddingService"] = None
    _model: Optional[SentenceTransformer] = None
    
    def __new__(cls):
        if cls._instance is None:
            cls._instance = super().__new__(cls)
        return cls._instance
    
    def __init__(self):
        if self._model is None:
            settings = get_settings()
            self._model = SentenceTransformer(settings.embedding_model)
            self.dimension = settings.embedding_dimension
    
    def embed(self, text: str) -> List[float]:
        """Generate embedding for a single text"""
        embedding = self._model.encode(text, normalize_embeddings=True)
        return embedding.tolist()
    
    def embed_batch(self, texts: List[str]) -> List[List[float]]:
        """Generate embeddings for multiple texts"""
        embeddings = self._model.encode(texts, normalize_embeddings=True)
        return embeddings.tolist()
    
    def similarity(self, embedding1: List[float], embedding2: List[float]) -> float:
        """Calculate cosine similarity between two embeddings"""
        vec1 = np.array(embedding1)
        vec2 = np.array(embedding2)
        return float(np.dot(vec1, vec2))
