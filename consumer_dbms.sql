CREATE TABLE IF NOT EXISTS consumer (
    name VARCHAR(50) PRIMARY KEY,
    password TEXT UNIQUE NOT NULL,
    roles TEXT[] NOT NULL
);

SELECT * FROM consumer ORDER BY name;
DROP TABLE IF EXISTS consumer;

CREATE TABLE IF NOT EXISTS consumer_image(
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL REFERENCES consumer(name) ON DELETE CASCADE,
    image_name VARCHAR(50),
    image_extension VARCHAR(50) NOT NULL,
    image_data BYTEA NOT NULL
);

SELECT * FROM consumer_image ORDER BY name;
DROP TABLE IF EXISTS consumer_image;