/// diesel table for stores
table! {
    stores (id) {
        id -> Integer,
        user_id -> Integer,
        is_active -> Bool,
        name -> Jsonb,
        short_description -> Jsonb,
        long_description -> Nullable<Jsonb>,
        slug -> VarChar,
        cover -> Nullable<VarChar>,
        logo -> Nullable<VarChar>,
        phone -> Nullable<VarChar>,
        email -> Nullable<VarChar>,
        address -> Nullable<VarChar>,
        facebook_url -> Nullable<VarChar>,
        twitter_url -> Nullable<VarChar>,
        instagram_url -> Nullable<VarChar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        default_language -> VarChar,
        slogan -> Nullable<VarChar>,
        rating -> Double,
        country -> Nullable<VarChar>,
        product_categories -> Nullable<Jsonb>,
        status -> VarChar,
        administrative_area_level_1 -> Nullable<VarChar>,
        administrative_area_level_2 -> Nullable<VarChar>,
        locality -> Nullable<VarChar>,
        political -> Nullable<VarChar>,
        postal_code -> Nullable<VarChar>,
        route -> Nullable<VarChar>,
        street_number -> Nullable<VarChar>,
        place_id -> Nullable<VarChar>,
    }
}

/// diesel table for wizard_stores
table! {
    wizard_stores (id) {
        id -> Integer,
        user_id -> Integer,
        store_id -> Nullable<Integer>,
        name -> Nullable<VarChar>,
        short_description -> Nullable<VarChar>,
        default_language -> Nullable<VarChar>,
        slug -> Nullable<VarChar>,
        country -> Nullable<VarChar>,
        address -> Nullable<VarChar>,
        administrative_area_level_1 -> Nullable<VarChar>,
        administrative_area_level_2 -> Nullable<VarChar>,
        locality -> Nullable<VarChar>,
        political -> Nullable<VarChar>,
        postal_code -> Nullable<VarChar>,
        route -> Nullable<VarChar>,
        street_number -> Nullable<VarChar>,
        place_id -> Nullable<VarChar>,
        completed -> Bool,
    }
}

/// diesel table for base_products
table! {
    base_products (id) {
        id -> Integer,
        is_active -> Bool,
        store_id -> Integer,
        name -> Jsonb,
        short_description -> Jsonb,
        long_description -> Nullable<Jsonb>,
        seo_title -> Nullable<Jsonb>,
        seo_description -> Nullable<Jsonb>,
        currency_id -> Integer,
        category_id -> Integer,
        views -> Integer,
        created_at -> Timestamp, // UTC 0, generated at db level
        updated_at -> Timestamp, // UTC 0, generated at db level
        rating -> Double,
        slug -> VarChar,
        status -> VarChar,
    }
}

/// diesel table for products
table! {
    products (id) {
        id -> Integer,
        base_product_id -> Integer,
        is_active -> Bool,
        discount -> Nullable<Double>,
        photo_main -> Nullable<VarChar>,
        additional_photos -> Nullable<Jsonb>,
        vendor_code -> VarChar,
        cashback -> Nullable<Double>,
        price -> Double,
        currency_id -> Nullable<Integer>,
        created_at -> Timestamp, // UTC 0, generated at db level
        updated_at -> Timestamp, // UTC 0, generated at db level
    }
}

joinable!(products -> base_products (base_product_id));
allow_tables_to_appear_in_same_query!(products, base_products);

/// diesel table for product attributes
table! {
    prod_attr_values (id) {
        id -> Integer,
        prod_id -> Integer,
        base_prod_id -> Integer,
        attr_id -> Integer,
        value -> VarChar,
        value_type -> VarChar,
        meta_field -> Nullable<VarChar>,
    }
}

table! {
    attributes {
        id -> Integer,
        name -> Jsonb,
        value_type -> VarChar,
        meta_field -> Nullable<Jsonb>,
    }
}

table! {
    cat_attr_values (id) {
        id -> Integer,
        cat_id -> Integer,
        attr_id -> Integer,
    }
}

table! {
    categories {
        id -> Integer,
        name -> Jsonb,
        meta_field -> Nullable<Jsonb>,
        parent_id -> Nullable<Integer>,
        level -> Integer,
    }
}

table! {
    currency_exchange (id) {
        id -> Integer,
        rouble -> Jsonb,
        euro -> Jsonb,
        dollar -> Jsonb,
        bitcoin -> Jsonb,
        etherium -> Jsonb,
        stq -> Jsonb,
        created_at -> Timestamp, // UTC 0, generated at db level
        updated_at -> Timestamp, // UTC 0, generated at db level
    }
}

/// diesel table for moderator_product_comments
table! {
    moderator_product_comments (id) {
        id -> Integer,
        moderator_id -> Integer,
        base_product_id -> Integer,
        comments -> VarChar,
        created_at -> Timestamp,
    }
}

/// diesel table for moderator_store_comments
table! {
    moderator_store_comments (id) {
        id -> Integer,
        moderator_id -> Integer,
        store_id -> Integer,
        comments -> VarChar,
        created_at -> Timestamp,
    }
}

table! {
    user_roles (id) {
        id -> Integer,
        user_id -> Integer,
        role -> VarChar,
    }
}
