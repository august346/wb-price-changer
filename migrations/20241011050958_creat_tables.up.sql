CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE suppliers (
  api_key UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  wb_id INTEGER,
  wb_jwt TEXT
);

CREATE TABLE products (
    id INTEGER PRIMARY KEY,
    price INTEGER NOT NULL,
    supplier_api_key UUID REFERENCES suppliers(api_key) ON DELETE CASCADE
);
