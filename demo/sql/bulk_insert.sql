-- Bulk insert script for testing batch processing capabilities
-- This script inserts 1000 products in a single transaction

BEGIN;

-- Generate 1000 products with varied categories and attributes
INSERT INTO products (name, description, price, category, brand, sku, in_stock, stock_quantity, tags, attributes, image_url)
SELECT 
    'Bulk Product ' || gs.id,
    'High-performance product from our bulk collection. Model #' || gs.id || ' with advanced features and premium quality.',
    ROUND((RANDOM() * 990 + 10)::NUMERIC, 2),
    CASE (gs.id % 8)
        WHEN 0 THEN 'Electronics'
        WHEN 1 THEN 'Fashion'
        WHEN 2 THEN 'Home & Garden'
        WHEN 3 THEN 'Books'
        WHEN 4 THEN 'Sports & Outdoors'
        WHEN 5 THEN 'Toys & Games'
        WHEN 6 THEN 'Beauty'
        ELSE 'Office'
    END,
    CASE (gs.id % 5)
        WHEN 0 THEN 'BulkBrand'
        WHEN 1 THEN 'MegaCorp'
        WHEN 2 THEN 'SuperStore'
        WHEN 3 THEN 'ValueMart'
        ELSE 'DirectSale'
    END,
    'BULK-' || TO_CHAR(gs.id, 'FM000000'),
    RANDOM() > 0.15,
    FLOOR(RANDOM() * 1000)::INTEGER,
    jsonb_build_array(
        'bulk',
        'batch-' || (gs.id / 100),
        CASE WHEN RANDOM() > 0.5 THEN 'popular' ELSE 'standard' END,
        CASE WHEN RANDOM() > 0.7 THEN 'featured' ELSE NULL END
    ) - 'null'::jsonb,
    jsonb_build_object(
        'batch_id', gs.id / 100,
        'weight', ROUND((RANDOM() * 20)::NUMERIC, 2) || 'kg',
        'dimensions', jsonb_build_object(
            'length', FLOOR(RANDOM() * 100 + 10),
            'width', FLOOR(RANDOM() * 100 + 10),
            'height', FLOOR(RANDOM() * 50 + 5)
        ),
        'warranty_months', CASE 
            WHEN gs.id % 10 = 0 THEN 24
            WHEN gs.id % 5 = 0 THEN 12
            ELSE 6
        END,
        'country_of_origin', CASE (gs.id % 6)
            WHEN 0 THEN 'USA'
            WHEN 1 THEN 'China'
            WHEN 2 THEN 'Germany'
            WHEN 3 THEN 'Japan'
            WHEN 4 THEN 'UK'
            ELSE 'Italy'
        END
    ),
    'https://example.com/products/bulk-' || gs.id || '.jpg'
FROM generate_series(10001, 11000) AS gs(id);

-- Add some products with special characteristics for testing

-- High-value electronics
INSERT INTO products (name, description, price, category, brand, sku, in_stock, stock_quantity, tags, attributes)
SELECT 
    'Premium ' || item_name,
    'Top-of-the-line ' || item_name || ' with cutting-edge technology',
    base_price + (RANDOM() * 1000),
    'Electronics',
    'TechPro',
    'PREM-' || UPPER(SUBSTRING(item_name, 1, 3)) || '-' || row_number,
    true,
    FLOOR(RANDOM() * 20 + 5)::INTEGER,
    '["premium", "high-end", "professional", "warranty"]'::jsonb,
    jsonb_build_object(
        'warranty_years', 3,
        'support_level', 'Premium',
        'certification', 'ISO 9001'
    )
FROM (
    VALUES 
        ('Laptop Pro', 2000, 1),
        ('Camera System', 3000, 2),
        ('Audio Interface', 1500, 3),
        ('Monitor 8K', 2500, 4),
        ('Workstation', 5000, 5)
) AS items(item_name, base_price, row_number);

-- Out of stock items for testing
INSERT INTO products (name, description, price, category, brand, sku, in_stock, stock_quantity, tags)
SELECT 
    'Limited Edition Item ' || gs.id,
    'Rare collectible item - currently out of stock',
    ROUND((RANDOM() * 400 + 100)::NUMERIC, 2),
    'Collectibles',
    'RareFinds',
    'RARE-' || TO_CHAR(gs.id, 'FM0000'),
    false,
    0,
    '["limited", "collectible", "out-of-stock", "rare"]'::jsonb
FROM generate_series(1, 20) AS gs(id);

-- Items with complex JSON attributes
INSERT INTO products (name, description, price, category, brand, sku, in_stock, stock_quantity, tags, attributes)
VALUES
(
    'Smart Home Hub Pro',
    'Advanced home automation system with AI integration',
    499.99,
    'Electronics',
    'SmartLife',
    'SMART-HUB-PRO',
    true,
    75,
    '["smart-home", "ai", "automation", "iot", "voice-control"]'::jsonb,
    '{
        "compatibility": ["Alexa", "Google Home", "HomeKit", "IFTTT"],
        "protocols": {
            "wifi": "802.11ac",
            "bluetooth": "5.0",
            "zigbee": true,
            "z-wave": true
        },
        "features": {
            "voice_control": true,
            "mobile_app": true,
            "scheduling": true,
            "energy_monitoring": true,
            "security_features": ["encryption", "2FA", "local_processing"]
        },
        "specifications": {
            "processor": "Quad-core ARM",
            "memory": "2GB RAM",
            "storage": "16GB",
            "ports": {
                "ethernet": 1,
                "usb": 2,
                "power": "USB-C"
            }
        }
    }'::jsonb
),
(
    'Professional Drone Kit',
    'Complete aerial photography solution',
    2499.99,
    'Electronics',
    'SkyTech',
    'DRONE-PRO-KIT',
    true,
    25,
    '["drone", "photography", "4k", "professional", "gps"]'::jsonb,
    '{
        "flight_specs": {
            "max_altitude": "500m",
            "flight_time": "35 minutes",
            "range": "10km",
            "max_speed": "72 km/h",
            "wind_resistance": "Level 5"
        },
        "camera": {
            "sensor": "1-inch CMOS",
            "resolution": "20MP",
            "video": ["4K/60fps", "1080p/120fps"],
            "gimbal": "3-axis",
            "zoom": "4x lossless"
        },
        "included_accessories": [
            "Extra batteries (2)",
            "Carrying case",
            "ND filters set",
            "Propeller guards",
            "Landing pad"
        ],
        "intelligent_features": [
            "Obstacle avoidance",
            "Follow me",
            "Waypoint navigation",
            "Return to home",
            "Gesture control"
        ]
    }'::jsonb
);

COMMIT;

-- Display summary
DO $$
DECLARE
    total_count INTEGER;
    bulk_count INTEGER;
    premium_count INTEGER;
    oos_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO total_count FROM products;
    SELECT COUNT(*) INTO bulk_count FROM products WHERE sku LIKE 'BULK-%';
    SELECT COUNT(*) INTO premium_count FROM products WHERE sku LIKE 'PREM-%';
    SELECT COUNT(*) INTO oos_count FROM products WHERE in_stock = false;
    
    RAISE NOTICE 'Bulk insert completed!';
    RAISE NOTICE 'Total products: %', total_count;
    RAISE NOTICE 'Bulk products added: %', bulk_count;
    RAISE NOTICE 'Premium products added: %', premium_count;
    RAISE NOTICE 'Out of stock products: %', oos_count;
END $$;