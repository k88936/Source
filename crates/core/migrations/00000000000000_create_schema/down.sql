-- Reverting drops every indexed folder, so the next index pass rebuilds it.
PRAGMA foreign_keys = OFF;
DROP TABLE assets;
DROP TABLE sync_state;
DROP TABLE folders;
PRAGMA foreign_keys = ON;
