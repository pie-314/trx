#!/usr/bin/env python3
"""
History module for tracking and managing package transaction history.
Provides transaction logging and rollback functionality.
"""

from .transaction_log import TransactionLog, TransactionEntry
from .rollback import rollback_transaction, confirm_rollback

__all__ = ['TransactionLog', 'TransactionEntry', 'rollback_transaction', 'confirm_rollback']