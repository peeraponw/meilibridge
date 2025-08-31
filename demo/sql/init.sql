-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_cron";

-- Create products table with CDC-friendly structure
CREATE TABLE products (
    id SERIAL PRIMARY KEY,
    external_id UUID DEFAULT uuid_generate_v4() UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    price DECIMAL(10, 2) NOT NULL,
    category VARCHAR(100),
    brand VARCHAR(100),
    sku VARCHAR(50) UNIQUE,
    in_stock BOOLEAN DEFAULT true,
    stock_quantity INTEGER DEFAULT 0,
    tags JSONB DEFAULT '[]'::jsonb,
    attributes JSONB DEFAULT '{}'::jsonb,
    image_url TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE DEFAULT NULL
);

-- Create index for soft deletes
CREATE INDEX idx_products_deleted_at ON products(deleted_at);

-- Create index for categories
CREATE INDEX idx_products_category ON products(category);

-- Create index for stock queries
CREATE INDEX idx_products_in_stock ON products(in_stock);

-- Create function to update the updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger to automatically update updated_at
CREATE TRIGGER update_products_updated_at BEFORE UPDATE ON products
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Create publication for CDC
CREATE PUBLICATION meilibridge_pub FOR TABLE products;

-- Insert sample data
INSERT INTO products (name, description, price, category, brand, sku, in_stock, stock_quantity, tags, attributes, image_url) VALUES
-- Electronics
('iPhone 15 Pro', 'Latest Apple smartphone with advanced features', 999.99, 'Electronics', 'Apple', 'APPL-IP15P-128', true, 50, '["smartphone", "apple", "5g", "premium"]'::jsonb, '{"color": "Titanium", "storage": "128GB", "display": "6.1 inch"}'::jsonb, 'https://example.com/iphone15pro.jpg'),
('Samsung Galaxy S24', 'Premium Android smartphone', 899.99, 'Electronics', 'Samsung', 'SAMS-GS24-256', true, 45, '["smartphone", "android", "5g", "samsung"]'::jsonb, '{"color": "Black", "storage": "256GB", "display": "6.2 inch"}'::jsonb, 'https://example.com/galaxys24.jpg'),
('MacBook Pro 16"', 'Professional laptop for creators', 2499.99, 'Electronics', 'Apple', 'APPL-MBP16-512', true, 20, '["laptop", "apple", "professional", "m3"]'::jsonb, '{"processor": "M3 Pro", "ram": "16GB", "storage": "512GB SSD"}'::jsonb, 'https://example.com/macbookpro16.jpg'),
('Sony WH-1000XM5', 'Premium noise-canceling headphones', 399.99, 'Electronics', 'Sony', 'SONY-WH1000XM5', true, 100, '["headphones", "wireless", "noise-canceling", "premium"]'::jsonb, '{"color": "Black", "battery": "30 hours", "type": "Over-ear"}'::jsonb, 'https://example.com/sonywh1000xm5.jpg'),
('iPad Air', 'Versatile tablet for work and play', 599.99, 'Electronics', 'Apple', 'APPL-IPAD-AIR5', true, 75, '["tablet", "apple", "portable", "m1"]'::jsonb, '{"color": "Space Gray", "storage": "64GB", "display": "10.9 inch"}'::jsonb, 'https://example.com/ipadair.jpg'),

-- Home & Garden
('Dyson V15 Vacuum', 'Advanced cordless vacuum cleaner', 699.99, 'Home & Garden', 'Dyson', 'DYSN-V15-DETECT', true, 30, '["vacuum", "cordless", "home", "cleaning"]'::jsonb, '{"type": "Cordless", "runtime": "60 minutes", "filtration": "HEPA"}'::jsonb, 'https://example.com/dysonv15.jpg'),
('Instant Pot Pro', 'Multi-functional pressure cooker', 129.99, 'Home & Garden', 'Instant Pot', 'INST-POT-PRO8', true, 150, '["kitchen", "cooking", "pressure-cooker", "multi-cooker"]'::jsonb, '{"capacity": "8 Quart", "programs": "10", "material": "Stainless Steel"}'::jsonb, 'https://example.com/instantpotpro.jpg'),
('Philips Hue Starter Kit', 'Smart lighting system', 199.99, 'Home & Garden', 'Philips', 'PHIL-HUE-START', true, 80, '["smart-home", "lighting", "led", "wifi"]'::jsonb, '{"bulbs": "4", "hub": "included", "colors": "16 million"}'::jsonb, 'https://example.com/philipshue.jpg'),

-- Fashion
('Nike Air Max 90', 'Classic sneakers', 129.99, 'Fashion', 'Nike', 'NIKE-AM90-BW', true, 200, '["shoes", "sneakers", "nike", "casual"]'::jsonb, '{"size": "Various", "color": "Black/White", "material": "Leather/Mesh"}'::jsonb, 'https://example.com/nikeairmax90.jpg'),
('Levi''s 501 Jeans', 'Original fit jeans', 79.99, 'Fashion', 'Levi''s', 'LEVI-501-ORIG', true, 300, '["jeans", "denim", "casual", "classic"]'::jsonb, '{"fit": "Original", "color": "Dark Blue", "material": "100% Cotton"}'::jsonb, 'https://example.com/levis501.jpg'),
('Ray-Ban Aviator', 'Classic aviator sunglasses', 169.99, 'Fashion', 'Ray-Ban', 'RAYB-AVTR-GOLD', true, 120, '["sunglasses", "accessories", "classic", "uv-protection"]'::jsonb, '{"frame": "Gold", "lens": "Green", "size": "58mm"}'::jsonb, 'https://example.com/raybanavtr.jpg'),

-- Books
('The Great Gatsby', 'Classic American novel', 14.99, 'Books', 'Scribner', 'BOOK-GATSBY-PB', true, 500, '["fiction", "classic", "american-literature", "1920s"]'::jsonb, '{"author": "F. Scott Fitzgerald", "pages": "180", "format": "Paperback"}'::jsonb, 'https://example.com/gatsby.jpg'),
('Atomic Habits', 'Guide to building good habits', 17.99, 'Books', 'Avery', 'BOOK-ATOMIC-HC', true, 400, '["self-help", "productivity", "habits", "bestseller"]'::jsonb, '{"author": "James Clear", "pages": "320", "format": "Hardcover"}'::jsonb, 'https://example.com/atomichabits.jpg'),

-- Sports & Outdoors
('Yeti Rambler 30oz', 'Insulated tumbler', 34.99, 'Sports & Outdoors', 'Yeti', 'YETI-RBLR-30OZ', true, 250, '["drinkware", "insulated", "outdoor", "travel"]'::jsonb, '{"capacity": "30oz", "color": "Navy", "insulation": "Double-wall vacuum"}'::jsonb, 'https://example.com/yetirambler.jpg'),
('Patagonia Down Jacket', 'Warm winter jacket', 279.99, 'Sports & Outdoors', 'Patagonia', 'PATG-DOWN-JKT', true, 60, '["jacket", "winter", "outdoor", "sustainable"]'::jsonb, '{"material": "Recycled Down", "waterproof": "DWR coating", "pockets": "3"}'::jsonb, 'https://example.com/patagoniadown.jpg'),

-- Toys & Games
('LEGO Creator Expert', 'Advanced building set', 179.99, 'Toys & Games', 'LEGO', 'LEGO-CRTR-EXP', true, 90, '["lego", "building", "creative", "expert"]'::jsonb, '{"pieces": "2354", "age": "16+", "theme": "Modular Building"}'::jsonb, 'https://example.com/legocreator.jpg'),
('Nintendo Switch OLED', 'Hybrid gaming console', 349.99, 'Toys & Games', 'Nintendo', 'NINT-SWCH-OLED', true, 40, '["gaming", "console", "nintendo", "portable"]'::jsonb, '{"display": "7-inch OLED", "storage": "64GB", "color": "White"}'::jsonb, 'https://example.com/switcholed.jpg'),

-- Beauty & Personal Care
('Olaplex Hair Treatment', 'Professional hair repair', 28.99, 'Beauty', 'Olaplex', 'OLPX-NO3-TREAT', true, 180, '["haircare", "treatment", "repair", "professional"]'::jsonb, '{"size": "100ml", "type": "Leave-in", "suitable": "All hair types"}'::jsonb, 'https://example.com/olaplex.jpg'),
('La Mer Moisturizer', 'Luxury face cream', 190.99, 'Beauty', 'La Mer', 'LMER-CRME-30ML', true, 50, '["skincare", "moisturizer", "luxury", "anti-aging"]'::jsonb, '{"size": "30ml", "type": "Face cream", "skin-type": "All"}'::jsonb, 'https://example.com/lamer.jpg'),

-- Office Supplies
('Herman Miller Aeron', 'Ergonomic office chair', 1395.99, 'Office', 'Herman Miller', 'HRMN-AERN-BLK', true, 15, '["chair", "ergonomic", "office", "premium"]'::jsonb, '{"size": "B", "color": "Graphite", "features": "PostureFit SL"}'::jsonb, 'https://example.com/aeron.jpg'),
('Standing Desk Converter', 'Adjustable desk riser', 299.99, 'Office', 'Varidesk', 'VARI-DESK-PRO', true, 70, '["desk", "standing", "ergonomic", "adjustable"]'::jsonb, '{"width": "36 inch", "height": "Adjustable", "capacity": "35 lbs"}'::jsonb, 'https://example.com/standingdesk.jpg');

-- Generate more sample products
DO $$
DECLARE
    i INTEGER;
    categories TEXT[] := ARRAY['Electronics', 'Home & Garden', 'Fashion', 'Books', 'Sports & Outdoors', 'Toys & Games', 'Beauty', 'Office'];
    brands TEXT[] := ARRAY['Generic', 'Premium', 'Value', 'Pro', 'Elite', 'Basic', 'Advanced', 'Ultimate'];
    product_types TEXT[] := ARRAY['Standard', 'Deluxe', 'Premium', 'Basic', 'Pro', 'Plus', 'Max', 'Ultra'];
BEGIN
    FOR i IN 1..980 LOOP
        INSERT INTO products (name, description, price, category, brand, sku, in_stock, stock_quantity, tags, attributes)
        VALUES (
            'Product ' || product_types[1 + (i % array_length(product_types, 1))] || ' ' || i,
            'High-quality product with excellent features and reliable performance. Item #' || i,
            (RANDOM() * 900 + 10)::DECIMAL(10, 2),
            categories[1 + (i % array_length(categories, 1))],
            brands[1 + (i % array_length(brands, 1))],
            'SKU-' || LPAD(i::TEXT, 6, '0'),
            RANDOM() > 0.1,
            FLOOR(RANDOM() * 500)::INTEGER,
            ('["tag' || (i % 10) || '", "tag' || (i % 7) || '", "featured"]')::jsonb,
            ('{"weight": "' || (RANDOM() * 10)::DECIMAL(4, 2) || 'kg", "warranty": "' || (1 + (i % 3)) || ' years"}')::jsonb
        );
    END LOOP;
END $$;

-- Create view for active products (non-deleted)
CREATE VIEW active_products AS
SELECT * FROM products WHERE deleted_at IS NULL;

-- Grant necessary permissions
GRANT SELECT ON products TO postgres;
GRANT USAGE ON SCHEMA public TO postgres;

-- Display initial counts
DO $$
DECLARE
    total_count INTEGER;
    active_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO total_count FROM products;
    SELECT COUNT(*) INTO active_count FROM products WHERE deleted_at IS NULL;
    RAISE NOTICE 'Initial data loaded: % total products, % active products', total_count, active_count;
END $$;