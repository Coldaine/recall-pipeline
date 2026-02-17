from functools import wraps
from typing import List, Optional

import numpy as np
import pytz
from sqlalchemy import func

from agents.constants import (
    MAX_EMBEDDING_DIM,
)
from agents.embeddings import embedding_model
from agents.orm.sqlite_functions import adapt_array
from agents.schemas.embedding_config import EmbeddingConfig
from agents.settings import settings


def build_query(
    base_query,
    search_field,
    query_text: Optional[str] = None,
    embedded_text: Optional[List[float]] = None,
    embed_query: bool = True,
    embedding_config: Optional[EmbeddingConfig] = None,
    ascending: bool = True,
    target_class: object = None,
):
    """
    Build a query based on the query text
    """

    if embed_query:
        if embedded_text is None:
            assert embedding_config is not None, (
                "embedding_config must be specified for vector search"
            )
            assert query_text is not None, (
                "query_text must be specified for vector search"
            )
            embedded_text = embedding_model(embedding_config).get_text_embedding(
                query_text
            )
            embedded_text = np.array(embedded_text)
            embedded_text = np.pad(
                embedded_text,
                (0, MAX_EMBEDDING_DIM - embedded_text.shape[0]),
                mode="constant",
            ).tolist()

    main_query = base_query.order_by(None)

    if embedded_text:
        # Check which database type we're using
        if settings.agents_pg_uri_no_default:
            # PostgreSQL with pgvector - use direct cosine_distance method
            if ascending:
                main_query = main_query.order_by(
                    search_field.cosine_distance(embedded_text).asc(),
                    target_class.created_at.asc(),
                    target_class.id.asc(),
                )
            else:
                main_query = main_query.order_by(
                    search_field.cosine_distance(embedded_text).asc(),
                    target_class.created_at.desc(),
                    target_class.id.asc(),
                )
        else:
            # SQLite with custom vector type
            query_embedding_binary = adapt_array(embedded_text)

            if ascending:
                main_query = main_query.order_by(
                    func.cosine_distance(search_field, query_embedding_binary).asc(),
                    target_class.created_at.asc(),
                    target_class.id.asc(),
                )
            else:
                main_query = main_query.order_by(
                    func.cosine_distance(search_field, query_embedding_binary).asc(),
                    target_class.created_at.desc(),
                    target_class.id.asc(),
                )

    else:
        # TODO: add other kinds of search
        raise NotImplementedError

    return main_query






