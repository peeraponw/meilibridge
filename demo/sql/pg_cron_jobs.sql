-- pg_cron scheduled jobs for MeiliBridge demo data generation
-- These jobs simulate realistic e-commerce activity

-- Ensure we're in the demo database
\c demo

-- Create a sequence for new product IDs
CREATE SEQUENCE IF NOT EXISTS demo_product_id_seq START WITH 2000;

-- Function to insert new products
CREATE OR REPLACE FUNCTION insert_new_product() RETURNS void AS $$
DECLARE
    product_id INTEGER;
    categories TEXT[] := ARRAY['Electronics', 'Fashion', 'Home & Garden', 'Books', 'Sports & Outdoors', 'Toys & Games', 'Beauty', 'Office'];
    brands TEXT[] := ARRAY['DemoBrand', 'AutoGen', 'CronCorp', 'ScheduledGoods', 'TimedProducts'];
    price DECIMAL(10,2);
    stock INTEGER;
    category TEXT;
    brand TEXT;
BEGIN
    product_id := nextval('demo_product_id_seq');
    category := categories[1 + floor(random() * array_length(categories, 1))::int];
    brand := brands[1 + floor(random() * array_length(brands, 1))::int];
    price := (random() * 490 + 10)::DECIMAL(10,2);
    stock := floor(random() * 200)::INTEGER;
    
    INSERT INTO products (name, description, price, category, brand, sku, in_stock, stock_quantity, tags, attributes)
    VALUES (
        'Auto Product ' || product_id,
        'Automatically generated product by pg_cron scheduler. Item #' || product_id,
        price,
        category,
        brand,
        'AUTO-' || LPAD(product_id::TEXT, 6, '0'),
        stock > 0,
        stock,
        jsonb_build_array('auto-generated', 'scheduled', lower(category), 'new'),
        jsonb_build_object(
            'generated_at', CURRENT_TIMESTAMP,
            'scheduler', 'pg_cron',
            'weight', round((random() * 10)::numeric, 2) || 'kg'
        )
    );
    
    RAISE LOG 'Inserted new product: Auto Product % in category % with price $%', product_id, category, price;
END;
$$ LANGUAGE plpgsql;

-- Function to update random product prices
CREATE OR REPLACE FUNCTION update_product_prices() RETURNS void AS $$
DECLARE
    updated_count INTEGER;
BEGIN
    WITH random_products AS (
        SELECT id 
        FROM products 
        WHERE deleted_at IS NULL 
        ORDER BY RANDOM() 
        LIMIT 5
    )
    UPDATE products p
    SET price = ROUND(p.price * (0.9 + random() * 0.2), 2),
        updated_at = CURRENT_TIMESTAMP
    FROM random_products rp
    WHERE p.id = rp.id;
    
    GET DIAGNOSTICS updated_count = ROW_COUNT;
    RAISE LOG 'Updated prices for % products', updated_count;
END;
$$ LANGUAGE plpgsql;

-- Function to update stock levels
CREATE OR REPLACE FUNCTION update_stock_levels() RETURNS void AS $$
DECLARE
    updated_count INTEGER;
BEGIN
    WITH random_products AS (
        SELECT id, stock_quantity
        FROM products 
        WHERE deleted_at IS NULL 
        ORDER BY RANDOM() 
        LIMIT 10
    )
    UPDATE products p
    SET stock_quantity = GREATEST(0, p.stock_quantity + floor(-50 + random() * 100)::INTEGER),
        in_stock = (p.stock_quantity + floor(-50 + random() * 100)::INTEGER) > 0,
        updated_at = CURRENT_TIMESTAMP
    FROM random_products rp
    WHERE p.id = rp.id;
    
    GET DIAGNOSTICS updated_count = ROW_COUNT;
    RAISE LOG 'Updated stock levels for % products', updated_count;
END;
$$ LANGUAGE plpgsql;

-- Function to add trending tags
CREATE OR REPLACE FUNCTION add_trending_tags() RETURNS void AS $$
DECLARE
    tags TEXT[] := ARRAY['trending', 'hot', 'popular', 'bestseller', 'featured', 'sale', 'limited'];
    tag TEXT;
    updated_count INTEGER;
BEGIN
    tag := tags[1 + floor(random() * array_length(tags, 1))::int];
    
    WITH random_products AS (
        SELECT id 
        FROM products 
        WHERE deleted_at IS NULL 
            AND NOT (tags ? tag)
        ORDER BY RANDOM() 
        LIMIT 3
    )
    UPDATE products p
    SET tags = tags || to_jsonb(tag),
        updated_at = CURRENT_TIMESTAMP
    FROM random_products rp
    WHERE p.id = rp.id;
    
    GET DIAGNOSTICS updated_count = ROW_COUNT;
    RAISE LOG 'Added tag "%" to % products', tag, updated_count;
END;
$$ LANGUAGE plpgsql;

-- Function to soft delete old products
CREATE OR REPLACE FUNCTION soft_delete_products() RETURNS void AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    UPDATE products
    SET deleted_at = CURRENT_TIMESTAMP,
        updated_at = CURRENT_TIMESTAMP
    WHERE id IN (
        SELECT id 
        FROM products 
        WHERE deleted_at IS NULL 
            AND created_at < CURRENT_TIMESTAMP - INTERVAL '1 hour'
        ORDER BY RANDOM() 
        LIMIT 2
    );
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RAISE LOG 'Soft deleted % old products', deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Function to restore deleted products
CREATE OR REPLACE FUNCTION restore_products() RETURNS void AS $$
DECLARE
    restored_count INTEGER;
BEGIN
    UPDATE products
    SET deleted_at = NULL,
        updated_at = CURRENT_TIMESTAMP,
        tags = tags || '["restored"]'::jsonb
    WHERE id IN (
        SELECT id 
        FROM products 
        WHERE deleted_at IS NOT NULL
        ORDER BY RANDOM() 
        LIMIT 1
    );
    
    GET DIAGNOSTICS restored_count = ROW_COUNT;
    RAISE LOG 'Restored % deleted products', restored_count;
END;
$$ LANGUAGE plpgsql;

-- Function for bulk category updates (simulating sales)
CREATE OR REPLACE FUNCTION category_sale() RETURNS void AS $$
DECLARE
    categories TEXT[] := ARRAY['Electronics', 'Fashion', 'Home & Garden'];
    category TEXT;
    discount DECIMAL(3,2);
    updated_count INTEGER;
BEGIN
    category := categories[1 + floor(random() * array_length(categories, 1))::int];
    discount := (0.1 + random() * 0.2)::DECIMAL(3,2); -- 10-30% discount
    
    UPDATE products
    SET price = ROUND(price * (1 - discount), 2),
        tags = CASE 
            WHEN tags ? 'sale' THEN tags 
            ELSE tags || '["sale"]'::jsonb 
        END,
        attributes = attributes || jsonb_build_object('discount_percent', (discount * 100)::INTEGER),
        updated_at = CURRENT_TIMESTAMP
    WHERE category = category
        AND deleted_at IS NULL
        AND price > 50;
    
    GET DIAGNOSTICS updated_count = ROW_COUNT;
    RAISE LOG 'Applied % discount to % products in % category', discount * 100, updated_count, category;
END;
$$ LANGUAGE plpgsql;

-- Function to generate complex transactions
CREATE OR REPLACE FUNCTION simulate_bulk_transaction() RETURNS void AS $$
BEGIN
    -- Start transaction
    -- Update multiple related products (e.g., buying a computer setup)
    UPDATE products
    SET stock_quantity = GREATEST(0, stock_quantity - 1),
        in_stock = stock_quantity > 1,
        updated_at = CURRENT_TIMESTAMP
    WHERE category = 'Electronics'
        AND deleted_at IS NULL
        AND in_stock = true
        AND id IN (
            SELECT id FROM products 
            WHERE category = 'Electronics' 
                AND deleted_at IS NULL 
                AND in_stock = true
            ORDER BY RANDOM() 
            LIMIT 3
        );
        
    RAISE LOG 'Simulated bulk transaction for Electronics bundle';
END;
$$ LANGUAGE plpgsql;

-- Schedule the jobs using pg_cron
-- Note: These will be scheduled after the database is initialized

-- Insert new products every minute
SELECT cron.schedule('insert-new-products', '* * * * *', 'SELECT insert_new_product();');

-- Update prices every minute
SELECT cron.schedule('update-prices', '* * * * *', 'SELECT update_product_prices();');

-- Update stock levels every minute
SELECT cron.schedule('update-stock', '* * * * *', 'SELECT update_stock_levels();');

-- Add trending tags every minute
SELECT cron.schedule('add-tags', '* * * * *', 'SELECT add_trending_tags();');

-- Soft delete products every 2 minutes
SELECT cron.schedule('soft-delete', '*/2 * * * *', 'SELECT soft_delete_products();');

-- Restore products every minute
SELECT cron.schedule('restore-products', '* * * * *', 'SELECT restore_products();');

-- Category sale every 5 minutes
SELECT cron.schedule('category-sale', '*/5 * * * *', 'SELECT category_sale();');

-- Bulk transactions every 3 minutes
SELECT cron.schedule('bulk-transaction', '*/3 * * * *', 'SELECT simulate_bulk_transaction();');

-- List all scheduled jobs
SELECT * FROM cron.job;

-- Create a view to monitor job execution
CREATE VIEW cron_job_stats AS
SELECT 
    j.jobid,
    j.jobname,
    j.schedule,
    j.command,
    COUNT(jh.runid) as total_runs,
    SUM(CASE WHEN jh.status = 'succeeded' THEN 1 ELSE 0 END) as successful_runs,
    SUM(CASE WHEN jh.status = 'failed' THEN 1 ELSE 0 END) as failed_runs,
    MAX(jh.start_time) as last_run,
    AVG(EXTRACT(EPOCH FROM (jh.end_time - jh.start_time))) as avg_duration_seconds
FROM cron.job j
LEFT JOIN cron.job_run_details jh ON j.jobid = jh.jobid
GROUP BY j.jobid, j.jobname, j.schedule, j.command;

-- Grant necessary permissions
GRANT USAGE ON SCHEMA cron TO postgres;
GRANT SELECT ON cron.job TO postgres;
GRANT SELECT ON cron.job_run_details TO postgres;

-- Create function to check demo activity
CREATE OR REPLACE FUNCTION get_demo_stats() RETURNS TABLE(
    metric TEXT,
    value BIGINT
) AS $$
BEGIN
    RETURN QUERY
    SELECT 'Total Products'::TEXT, COUNT(*)::BIGINT FROM products
    UNION ALL
    SELECT 'Active Products'::TEXT, COUNT(*)::BIGINT FROM products WHERE deleted_at IS NULL
    UNION ALL
    SELECT 'Out of Stock'::TEXT, COUNT(*)::BIGINT FROM products WHERE in_stock = false AND deleted_at IS NULL
    UNION ALL
    SELECT 'On Sale'::TEXT, COUNT(*)::BIGINT FROM products WHERE tags ? 'sale' AND deleted_at IS NULL
    UNION ALL
    SELECT 'Products Added Today'::TEXT, COUNT(*)::BIGINT FROM products WHERE created_at::date = CURRENT_DATE;
END;
$$ LANGUAGE plpgsql;

-- Log initial setup complete
DO $$
BEGIN
    RAISE NOTICE 'pg_cron jobs scheduled successfully!';
    RAISE NOTICE 'Data generation will start automatically.';
    RAISE NOTICE 'View job status: SELECT * FROM cron_job_stats;';
    RAISE NOTICE 'View demo stats: SELECT * FROM get_demo_stats();';
END $$;