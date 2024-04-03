# Music Library

This is a music catalog system, where the information is stored in a MariaDB database and accessed via a TUI. You can add, remove, modify, and search for songs, artists, and albums in your library. This program was written in Rust with [SQLx](https://crates.io/crates/sqlx) for the database integration and [Ratatui](https://ratatui.rs/) for the UI. 

## Completed
- Database integration works, queries work ✅
- All CRUD operations tested ✅
- `-h/--help` and `-v/--version` CLI args work ✅
- TUI displays songs from database ✅
- TUI search works ✅
- New Song works ✅
- Edit Song works ✅
- Delete Song works ✅

All project features are completed! 

I will continue testing the UI and fixing bugs as I find them, but in the general case this application is complete. 

## To Run This Application: 

### Dependencies
In order to run this application, you need to have these dependencies: 

 - [Cargo](https://www.rust-lang.org/tools/install)
 - [MariaDB/MySQL](https://mariadb.org/download/)
 - [sqlx](https://crates.io/crates/sqlx)
 - [sqlx-cli](https://crates.io/crates/sqlx-cli)
 - [Ratatui](https://crates.io/crates/ratatui/)
 - [Crossterm](https://crates.io/crates/crossterm)

 Once everything is installed and/or compiled, you need to initialize the database. 

### Initializing the Database

In your preferred SQL editor (I use DBeaver), use the included [schema](schema.sql) to initialize the tables. 

* note: You must first create a database called `music` before running the schema. 

### Building and Running the App

Now, you should navigate in your terminal to the directory where you downloaded this source code and  use the command

```
$ cargo sqlx prepare -D mysql://<Database Username>:<Database Password>@localhost:<Your Database Port>/music
```
* the usual port for a MySQL database is port 3306

This will run sqlx on the code to ensure that all of the queries will work. Sqlx checks all queries at compile-time and saves them in a  `.sqlx/` directory. This way, sqlx can protect your queries from SQL injection and it will ensure that they work before you run your code. 

Now that your SQL queries are prepared, you should be all clear to run the code: 

 ```
 $ cargo run
 ```

If you installed your dependencies correctly, the application will begin to compile. If you set up and prepared your database correctly, it will run too. 

The TUI provides instructions at the bottom of the screen, but I suppose I should make a user guide eventually. 

Please submit an [issue](https://github.com/kcajeel/music-library/issues) if you encounter any errors or need any clarification. 