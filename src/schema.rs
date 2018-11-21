table! {
    attributes (id) {
        id -> Int4,
        name -> Jsonb,
        value_type -> Varchar,
        meta_field -> Nullable<Jsonb>,
        uuid -> Uuid,
    }
}

table! {
    attribute_values (id) {
        id -> Int4,
        attr_id -> Int4,
        code -> Varchar,
        translations -> Nullable<Jsonb>,
    }
}

table! {
    base_products (id) {
        id -> Int4,
        store_id -> Int4,
        is_active -> Bool,
        name -> Jsonb,
        short_description -> Jsonb,
        long_description -> Nullable<Jsonb>,
        category_id -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        views -> Int4,
        seo_title -> Nullable<Jsonb>,
        seo_description -> Nullable<Jsonb>,
        rating -> Float8,
        slug -> Varchar,
        status -> Varchar,
        kafka_update_no -> Int4,
        currency -> Varchar,
        uuid -> Uuid,
    }
}

table! {
    cat_attr_values (id) {
        id -> Int4,
        cat_id -> Int4,
        attr_id -> Int4,
    }
}

table! {
    categories (id) {
        id -> Int4,
        name -> Jsonb,
        parent_id -> Nullable<Int4>,
        level -> Int4,
        meta_field -> Nullable<Jsonb>,
        is_active -> Bool,
        uuid -> Uuid,
    }
}

table! {
    coupons (id) {
        id -> Int4,
        code -> Varchar,
        title -> Varchar,
        store_id -> Int4,
        scope -> Varchar,
        percent -> Int4,
        quantity -> Int4,
        expired_at -> Nullable<Timestamp>,
        is_active -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    coupon_scope_base_products (id) {
        id -> Int4,
        coupon_id -> Int4,
        base_product_id -> Int4,
    }
}

table! {
    coupon_scope_categories (id) {
        id -> Int4,
        coupon_id -> Int4,
        category_id -> Int4,
    }
}

table! {
    currency_exchange (id) {
        id -> Uuid,
        data -> Jsonb,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    custom_attributes (id) {
        id -> Int4,
        base_product_id -> Int4,
        attribute_id -> Int4,
    }
}

table! {
    moderator_product_comments (id) {
        id -> Int4,
        moderator_id -> Int4,
        base_product_id -> Int4,
        comments -> Varchar,
        created_at -> Timestamp,
    }
}

table! {
    moderator_store_comments (id) {
        id -> Int4,
        moderator_id -> Int4,
        store_id -> Int4,
        comments -> Varchar,
        created_at -> Timestamp,
    }
}

table! {
    prod_attr_values (id) {
        id -> Int4,
        prod_id -> Int4,
        attr_id -> Int4,
        value -> Varchar,
        value_type -> Varchar,
        meta_field -> Nullable<Varchar>,
        base_prod_id -> Int4,
        attr_value_id -> Nullable<Int4>,
    }
}

table! {
    products (id) {
        id -> Int4,
        is_active -> Bool,
        discount -> Nullable<Float8>,
        photo_main -> Nullable<Varchar>,
        cashback -> Nullable<Float8>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        base_product_id -> Int4,
        additional_photos -> Nullable<Jsonb>,
        price -> Float8,
        vendor_code -> Varchar,
        currency -> Varchar,
        kafka_update_no -> Int4,
        pre_order -> Bool,
        pre_order_days -> Int4,
        uuid -> Uuid,
    }
}

table! {
    stores (id) {
        id -> Int4,
        user_id -> Int4,
        is_active -> Bool,
        slug -> Varchar,
        cover -> Nullable<Varchar>,
        logo -> Nullable<Varchar>,
        phone -> Nullable<Varchar>,
        email -> Nullable<Varchar>,
        address -> Nullable<Varchar>,
        facebook_url -> Nullable<Varchar>,
        twitter_url -> Nullable<Varchar>,
        instagram_url -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        slogan -> Nullable<Varchar>,
        default_language -> Varchar,
        name -> Jsonb,
        short_description -> Jsonb,
        long_description -> Nullable<Jsonb>,
        rating -> Float8,
        country -> Nullable<Varchar>,
        product_categories -> Nullable<Jsonb>,
        status -> Varchar,
        administrative_area_level_1 -> Nullable<Varchar>,
        administrative_area_level_2 -> Nullable<Varchar>,
        locality -> Nullable<Varchar>,
        political -> Nullable<Varchar>,
        postal_code -> Nullable<Varchar>,
        route -> Nullable<Varchar>,
        street_number -> Nullable<Varchar>,
        place_id -> Nullable<Varchar>,
        kafka_update_no -> Int4,
        country_code -> Nullable<Varchar>,
        uuid -> Uuid,
    }
}

table! {
    used_coupons (coupon_id, user_id) {
        coupon_id -> Int4,
        user_id -> Int4,
    }
}

table! {
    user_roles (id) {
        user_id -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        name -> Varchar,
        data -> Nullable<Jsonb>,
        id -> Uuid,
    }
}

table! {
    wizard_stores (id) {
        id -> Int4,
        user_id -> Int4,
        store_id -> Nullable<Int4>,
        name -> Nullable<Varchar>,
        short_description -> Nullable<Varchar>,
        default_language -> Nullable<Varchar>,
        slug -> Nullable<Varchar>,
        country -> Nullable<Varchar>,
        address -> Nullable<Varchar>,
        administrative_area_level_1 -> Nullable<Varchar>,
        administrative_area_level_2 -> Nullable<Varchar>,
        locality -> Nullable<Varchar>,
        political -> Nullable<Varchar>,
        postal_code -> Nullable<Varchar>,
        route -> Nullable<Varchar>,
        street_number -> Nullable<Varchar>,
        place_id -> Nullable<Varchar>,
        completed -> Bool,
        country_code -> Nullable<Varchar>,
    }
}

joinable!(attribute_values -> attributes (attr_id));
joinable!(base_products -> categories (category_id));
joinable!(base_products -> stores (store_id));
joinable!(cat_attr_values -> attributes (attr_id));
joinable!(cat_attr_values -> categories (cat_id));
joinable!(coupon_scope_base_products -> base_products (base_product_id));
joinable!(coupon_scope_base_products -> coupons (coupon_id));
joinable!(coupon_scope_categories -> categories (category_id));
joinable!(coupon_scope_categories -> coupons (coupon_id));
joinable!(coupons -> stores (store_id));
joinable!(custom_attributes -> attributes (attribute_id));
joinable!(custom_attributes -> base_products (base_product_id));
joinable!(moderator_product_comments -> base_products (base_product_id));
joinable!(moderator_store_comments -> stores (store_id));
joinable!(prod_attr_values -> attribute_values (attr_value_id));
joinable!(prod_attr_values -> attributes (attr_id));
joinable!(prod_attr_values -> base_products (base_prod_id));
joinable!(prod_attr_values -> products (prod_id));
joinable!(products -> base_products (base_product_id));
joinable!(used_coupons -> coupons (coupon_id));

allow_tables_to_appear_in_same_query!(
    attributes,
    attribute_values,
    base_products,
    cat_attr_values,
    categories,
    coupons,
    coupon_scope_base_products,
    coupon_scope_categories,
    currency_exchange,
    custom_attributes,
    moderator_product_comments,
    moderator_store_comments,
    prod_attr_values,
    products,
    stores,
    used_coupons,
    user_roles,
    wizard_stores,
);
