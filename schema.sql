-- Including my SQL schema so the database can be replicated

USE music;

CREATE table if not exists Artists(
  name VARCHAR(60) not null PRIMARY KEY
);
  
CREATE TABLE if not exists Albums(
  title VARCHAR(60) not null,
  artist VARCHAR(200) not null,
  release_year INT not null,
  media_type VARCHAR(60) not null,
  PRIMARY KEY(title,artist,release_year,media_type));
    
CREATE TABLE if not exists Songs(
  if int unsigned not null auto_increment primary key,
  title VARCHAR(60) not null,
  artist VARCHAR(200) not null,
  album VARCHAR(1000) not null,
  release_year INT not null,
  media_type VARCHAR(60)) not null;