from typing import Optional

from sqlalchemy import String, Text
from sqlalchemy.orm import Mapped, mapped_column

from agents.orm.sqlalchemy_base import SqlalchemyBase


class Project(SqlalchemyBase):
    """Represents a logical project grouping activities."""

    __tablename__ = "projects"

    id: Mapped[str] = mapped_column(String, primary_key=True)
    name: Mapped[Optional[str]] = mapped_column(String, nullable=True, doc="Inferred or manual name")
    summary: Mapped[Optional[str]] = mapped_column(Text, nullable=True, doc="Project summary")
