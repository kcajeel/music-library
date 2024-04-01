# Music Library

This is a music catalog system, where the information is stored in a MariaDB database and accessed via a TUI. You can add, remove, modify, and search for songs, artists, and albums in your library. This program was written in Rust with [SQLx](https://crates.io/crates/sqlx) for the database integration and [Ratatui](https://ratatui.rs/) for the UI. 

## Completed
- Database integration works, queries work ✅
- All CRUD operations tested ✅
- `-h/--help` and `-v/--version` CLI args work ✅
- TUI displays songs from database ✅
- TUI search works ✅

## TODO
- TUI add/edit/delete song popups don't functionally do anything yet