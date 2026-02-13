from datetime import date
from typing import Optional

from sqlalchemy import Date, String, Text
from sqlalchemy.orm import Mapped, mapped_column

from agents.orm.sqlalchemy_base import SqlalchemyBase


class DaySummary(SqlalchemyBase):
    """Daily summary, optionally per-deployment or cross-deployment."""

    __tablename__ = "day_summaries"

    id: Mapped[str] = mapped_column(String, primary_key=True)
    deployment_id: Mapped[Optional[str]] = mapped_column(
        String, nullable=True, doc="NULL for cross-deployment summary"
    )
    date: Mapped[date] = mapped_column(Date, nullable=False)
    summary: Mapped[Optional[str]] = mapped_column(Text, nullable=True)
