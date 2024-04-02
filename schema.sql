-- Including my SQL schema so the database can be replicated

# USE music;

CREATE table if not exists Artists(
  name VARCHAR(60) not null PRIMARY KEY
);
  
CREATE TABLE if not exists Albums(
  id int unsigned not null auto_increment primary key,
  title VARCHAR(60) not null,
  artist VARCHAR(200) not null,
  release_year INT not null,
  media_type VARCHAR(60) not null);

CREATE TABLE if not exists Songs(
  id int unsigned not null auto_increment primary key,
  title VARCHAR(60) not null,
  artist VARCHAR(200) not null,
  album VARCHAR(1000) not null,
  release_year INT not null,
  media_type VARCHAR(60) not null);

  
INSERT INTO Artists VALUES
    ('Radiohead'),
    ('Future'),
    ('Bob Dylan'),
    ('2 Chainz'),
    ('Black Sabbath'),
    ('The Beatles'),
    ('B.B. King'),
    ('Chief Keef');
    
INSERT INTO Albums VALUES
    ('The Bends','Radiohead',1995,'Digital Download'),
    ('HNDRXX', 'Future', 2017, 'Digital Download'),
    ('The Freewheelin Bob Dylan', 'Bob Dylan', 1963, 'Vinyl'),
    ('Pretty Girls Like Trap Music', '2 Chainz', 2017, 'Digital Download'),
    ('Paranoid', 'Black Sabbath', 1970, 'Vinyl'),
    ('Magical Mystery Tour', 'The Beatles', 1967, 'Vinyl'),
    ('B.B. King in London', 'B.B. King', 1971, 'Vinyl'),
    ('Finally Rich', 'Chief Keef', 2012, 'Digital Download');
    
INSERT INTO Songs VALUES
    (0, 'My Iron Lung', 'Radiohead', 'The Bends', 1995, 'Digital Download'),
    (0, 'Solo', 'Future', 'HNDRXX', 2017, 'Digital Download'),
    (0, 'Masters of War', 'Bob Dylan', 'The Freewheelin Bob Dylan', 1963, 'Vinyl'),
    (0, 'Riverdale Rd', '2 Chainz', 'Pretty Girls Like Trap Music', 2017, 'Digital Download'),
    (0, 'War Pigs', 'Black Sabbath', 'Paranoid', 1970, 'Vinyl'),
    (0, 'I Am the Walrus', 'The Beatles', 'Magical Mystery Tour', 1967, 'Vinyl'),
    (0, 'Caldonia', 'B.B. King', 'B.B. King in London', 1971, 'Vinyl'),
    (0, 'Love Sosa', 'Chief keef', 'Finally Rich', 2012, 'Digital Download');
    