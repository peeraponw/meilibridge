-- PostgreSQL initialization script for MeiliBridge development

-- Create replication user
CREATE USER replication_user WITH REPLICATION LOGIN PASSWORD 'replication_password';

-- Grant necessary permissions
GRANT USAGE ON SCHEMA public TO replication_user;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO replication_user;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO replication_user;

-- Create sample tables for testing
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    age INTEGER,
    active BOOLEAN DEFAULT true,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS products (
    sku VARCHAR(50) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    price DECIMAL(10, 2),
    category VARCHAR(100),
    tags TEXT[],
    in_stock BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id INTEGER REFERENCES users(id),
    status VARCHAR(50) NOT NULL,
    total_amount DECIMAL(10, 2),
    items JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create update timestamp trigger
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_products_updated_at BEFORE UPDATE ON products
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_orders_updated_at BEFORE UPDATE ON orders
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Create publication for all tables
CREATE PUBLICATION meilibridge_pub FOR ALL TABLES;

-- Insert some sample data
INSERT INTO users (email, first_name, last_name, age, metadata) VALUES
    ('john.doe@example.com', 'John', 'Doe', 30, '{"interests": ["technology", "sports"]}'),
    ('jane.smith@example.com', 'Jane', 'Smith', 25, '{"interests": ["music", "travel"]}'),
    ('bob.wilson@example.com', 'Bob', 'Wilson', 35, '{"interests": ["cooking", "photography"]}');

INSERT INTO products (sku, name, description, price, category, tags) VALUES
    ('LAPTOP-001', 'ThinkPad X1 Carbon', 'High-performance business laptop', 1599.99, 'Electronics', ARRAY['laptop', 'business', 'premium']),
    ('PHONE-001', 'iPhone 15 Pro', 'Latest flagship smartphone', 1199.99, 'Electronics', ARRAY['phone', 'smartphone', 'premium']),
    ('BOOK-001', 'Clean Code', 'A Handbook of Agile Software Craftsmanship', 49.99, 'Books', ARRAY['programming', 'software', 'education']);

-- Grant permissions on new tables
GRANT SELECT ON users, products, orders TO replication_user;

-- Display connection info
\echo 'PostgreSQL setup complete!'
\echo 'Connection details:'
\echo '  Host: localhost (or postgres within Docker network)'
\echo '  Port: 5433 (external) / 5432 (internal)'
\echo '  Database: meilibridge_dev'
\echo '  User: postgres'
\echo '  Password: meilibridge_dev_password'
\echo '  Replication User: replication_user'
\echo '  Replication Password: replication_password'