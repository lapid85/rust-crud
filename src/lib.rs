extern crate inflector;

use inflector::cases::snakecase;
use inflector::string::pluralize;
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// 自动实现 crud
#[proc_macro_derive(CRUDTable)]
pub fn impl_crud_table(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input_clone = input.clone();
    let input_clone2 = input.clone();

    let mut tokens: Vec<proc_macro2::TokenStream> = Vec::new();
    tokens.push(quote! {
        pub fn new() -> Self {
            Self::default()
        }
    });
    {
        let DeriveInput { ident, data, .. } = parse_macro_input!(input);
        let struct_type = ident.to_string();
        let snake_case = snakecase::to_snake_case(&struct_type);
        let table_name = pluralize::to_plural(&snake_case);
        tokens.push(quote! {

            #[inline]
            pub fn get_table_name() -> &'static str {
                #table_name
            }

            /// get_all: 获取全部记录
            pub async fn get_all(pool: &common::types::Db) -> Result<Vec<Self>, &'static str> {
                let sql = format!("SELECT {} FROM {}", Self::get_fields(), Self::get_table_name());
                sqlx::query_as::<_, Self>(&sql).fetch_all(pool).await.map_err(|e| {
                        println!("get_all error: {:?}", e);
                        "获取数据失败"
                })
            }

            /// get_all_by_cond: 获取带分页的全部记录
            pub async fn get_all_by_cond(pool: &common::types::Db, cond: &common::types::Cond) -> Result<(Vec<Self>, i64), &'static str> {
                let sql_cond = cond.build();
                let where_str = if cond.has_args() { format!("WHERE {}", &sql_cond) } else { sql_cond.to_owned() };
                let sql = format!("SELECT {} FROM {} {}", Self::get_fields(), Self::get_table_name(), where_str);
                println!("SQL: {}", sql);
                let where_str_total = if cond.has_args() { format!("WHERE {}", &sql_cond) } else { "".to_string() };
                let sql_total = format!("SELECT COUNT(*) AS total FROM {} {}", Self::get_table_name(), where_str_total);
                println!("SQL_TOTAL: {}", sql_total);
                let mut builder = sqlx::query_as::<_, Self>(&sql);
                let mut builder_total = sqlx::query_as::<_, common::types::pg::Total>(&sql_total);
                for v in &cond.args {
                    match v {
                        common::types::Val::I8(rv) =>   { builder = builder.bind::<i8>(*rv);            builder_total = builder_total.bind::<i8>(*rv); },
                        common::types::Val::U8(rv) =>   { builder = builder.bind::<i8>(*rv as i8);      builder_total = builder_total.bind::<i8>(*rv as i8); },
                        common::types::Val::I16(rv) =>  { builder = builder.bind::<i16>(*rv);           builder_total = builder_total.bind::<i16>(*rv); },
                        common::types::Val::U16(rv) =>  { builder = builder.bind::<i16>(*rv as i16);    builder_total = builder_total.bind::<i16>(*rv as i16); },
                        common::types::Val::I32(rv) =>  { builder = builder.bind::<i32>(*rv);           builder_total = builder_total.bind::<i32>(*rv); },
                        common::types::Val::U32(rv) =>  { builder = builder.bind::<i32>(*rv as i32);    builder_total = builder_total.bind::<i32>(*rv as i32); },
                        common::types::Val::I64(rv) =>  { builder = builder.bind::<i64>(*rv);           builder_total = builder_total.bind::<i64>(*rv); },
                        common::types::Val::U64(rv) =>  { builder = builder.bind::<i64>(*rv as i64);    builder_total = builder_total.bind::<i64>(*rv as i64); },
                        common::types::Val::F32(rv) =>  { builder = builder.bind::<f32>(*rv);           builder_total = builder_total.bind::<f32>(*rv); },
                        common::types::Val::F64(rv) =>  { builder = builder.bind::<f64>(*rv);           builder_total = builder_total.bind::<f64>(*rv); },
                        common::types::Val::Str(rv) =>  { builder = builder.bind(rv);                   builder_total = builder_total.bind(rv); },
                        common::types::Val::S(rv) =>    { builder = builder.bind(rv);                   builder_total = builder_total.bind(rv); }, 
                        common::types::Val::Bool(rv) => { builder = builder.bind(rv);                   builder_total = builder_total.bind(rv); }, 
                            _ => { continue; }
                    };
                }
                let rows = match builder.fetch_all(pool).await {
                    Ok(v) => v,
                    Err(err) => {
                        println!("get_all_by_cond error: {:?}", err);
                        return Err("获取数据失败: get_all_by_cond - select");
                    }
                };
                
                let rows_total = match builder_total.fetch_one(pool).await {
                    Ok(v) => v,
                    Err(err) => {
                        println!("get_all_and_count_order_by error: {:?}", err);
                        return Err("获取数据失败: get_all_and_count_order_by - count");
                    }
                };
                // pool.close().await;
                Ok((rows, rows_total.total))
            }

            /// get_all_by_query: 获取按查询条件/分页的全部记录 - 可以把 Order by 写到 query 查询条件里面
            pub async fn get_all_by_query(pool: &common::types::Db, query: &str, values: &[common::types::Val]) -> Result<Vec<Self>, &'static str> {
                let query_cond = if query.is_empty() { String::from("") } else { format!("WHERE {}", query) };
                let sql = format!("SELECT {} FROM {} {}", Self::get_fields(), Self::get_table_name(), &query_cond);
                let mut builder = sqlx::query_as::<_, Self>(&sql);
                for v in values {
                    match v {
                        common::types::Val::I8(rv)      => { builder = builder.bind::<i8>(*rv); },
                        common::types::Val::U8(rv)      => { builder = builder.bind::<i8>(*rv as i8); },
                        common::types::Val::I16(rv)     => { builder = builder.bind::<i16>(*rv); },
                        common::types::Val::U16(rv)     => { builder = builder.bind::<i16>(*rv as i16); },
                        common::types::Val::I32(rv)     => { builder = builder.bind::<i32>(*rv); },
                        common::types::Val::U32(rv)     => { builder = builder.bind::<i32>(*rv as i32); },
                        common::types::Val::I64(rv)     => { builder = builder.bind::<i64>(*rv); },
                        common::types::Val::U64(rv)     => { builder = builder.bind::<i64>(*rv as i64); },
                        common::types::Val::F32(rv)     => { builder = builder.bind::<f32>(*rv); },
                        common::types::Val::F64(rv)     => { builder = builder.bind::<f64>(*rv); },
                        common::types::Val::Str(rv)     => { builder = builder.bind(rv); },
                        common::types::Val::S(rv)       => { builder = builder.bind(rv); },
                        common::types::Val::Bool(rv)    => { builder = builder.bind(rv); },
                        _ => { continue; }
                    };
                }
                let rows = match builder.fetch_all(pool).await {
                    Ok(v) => v,
                    Err(err) => {
                        println!("SQL: {}\n{}", sql, err);
                        return Err("获取数据失败: 无法依据条件获取数据");
                    }
                };
                Ok(rows)
            }

            /// get_all_by_query_raw: 获取全部记录
            pub async fn get_all_by_query_raw(pool: &common::types::Db, sql: &str) -> Result<Vec<Self>, &'static str> {
                sqlx::query_as::<_, Self>(sql).fetch_all(pool).await.map_err(|e| {
                        println!("get_all error: {:?}", e);
                        "获取数据失败"
                })
            }

            /// get_by_cond: 查询单条记录 - 依据条件
            pub async fn get_by_cond(pool: &common::types::Db, cond: &common::types::Cond) -> Option<Self> {
                let sql_cond = cond.build();
                let where_str = if cond.has_args() { format!("WHERE {}", &sql_cond) } else { sql_cond };
                let sql = format!("SELECT {} FROM {} {}", Self::get_fields(), Self::get_table_name(), where_str);
                if let Ok(v) = sqlx::query_as::<_, Self>(&sql).fetch_one(pool).await {
                    return Some(v);
                }
                None
            }

            /// get_by_query: 获取按查询条件/分页的单条记录 - 可以把 Order by 写到 query 查询条件里面
            pub async fn get_by_query(pool: &common::types::Db, query: &str, values: &[common::types::Val]) -> Option<Self> {
                if let Ok(mut rows) = Self::get_all_by_query(pool, query, values).await {
                    return rows.pop();
                }
                None
            }

            /// get_by_query_raw: 查询单条记录 - 原始sql
            pub async fn get_by_query_raw(pool: &common::types::Db, sql: &str) -> Option<Self> {
                if let Ok(v) = sqlx::query_as::<_, Self>(sql).fetch_one(pool).await {
                    return Some(v);
                }
                None
            }
        });
        match data {
            Data::Struct(s) => match s.fields {
                Fields::Named(f) => {
                    for field in f.named {
                        let field_name = field.ident.as_ref().unwrap();
                        let field_type = field.ty;

                        // get_by_# 方法
                        let current_field = format!("{}", field_name); // 当前字段名称
                        let get_by_method = Ident::new(
                            &format!("get_by_{}", field_name),
                            proc_macro2::Span::call_site(),
                        );
                        tokens.push(quote!{
                            /// 依据字段 #current_field 得到单条记录
                            pub async fn #get_by_method(pool: &common::types::Db, field_value: &#field_type) -> Result<Self, &'static str> {
                                let sql = format!("SELECT {} FROM {} WHERE {} = $1 LIMIT 1", Self::get_fields(), Self::get_table_name(), #current_field);
                                sqlx::query_as::<_, Self>(&sql).bind(field_value).fetch_one(pool).await.map_err(|e| {
                                        println!("{}", e);
                                        "获取数据失败"
                                    })
                            }
                        });

                        // get_all_by_# 方法
                        let get_all_by_method = Ident::new(
                            &format!("get_all_by_{}", field_name),
                            proc_macro2::Span::call_site(),
                        );
                        let sql_where = format!("WHERE {} = $1", field_name.to_owned());
                        tokens.push(quote!{
                            /// 依据字段 #current_field 得到所有记录
                            pub async fn #get_all_by_method(pool: &common::types::Db, field_value: &#field_type) -> Result<Vec<Self>, &'static str> {
                                let sql = format!("SELECT {} FROM {} {}", Self::get_fields(), Self::get_table_name(), #sql_where);
                                sqlx::query_as::<_, Self>(&sql).bind(field_value).fetch_all(pool).await.map_err(|e| {
                                        println!("{}", e);
                                        "获取数据失败"
                                    })
                            }
                        });

                        // update_by
                        let update_by_method = Ident::new(
                            &format!("update_by_{}", field_name),
                            proc_macro2::Span::call_site(),
                        );
                        let update_where_sql = format!(
                            "SET {} = $1 WHERE {} = $2 LIMIT 1",
                            &field_name, &field_name
                        );
                        tokens.push(quote!{
                            /// 依据字段 #current_field 更新单条记录
                            pub async fn #update_by_method(pool: &common::types::Db, value_old: &#field_type, value_new: &#field_type) ->Result<(), &'static str> {
                                let sql = format!("UPDATE {} {} LIMIT 1", Self::get_table_name(), #update_where_sql);
                                match sqlx::query(&sql).bind(value_old).bind(value_new).execute(pool).await {
                                        Ok(_) => Ok(()),
                                        Err(err) => {
                                            println!("{}", err);
                                            Err("更新数据失败")
                                        }
                                }
                            }
                        });

                        // update_all_by
                        let update_all_by_method = Ident::new(
                            &format!("update_all_by_{}", field_name),
                            proc_macro2::Span::call_site(),
                        );
                        let update_all_where_sql =
                            format!("SET {} = $1 WHERE {} = $2", &field_name, &field_name);
                        tokens.push(quote!{
                            /// 依据字段 #current_field 更新单条记录
                            pub async fn #update_all_by_method(pool: &common::types::Db, value_old: &#field_type, value_new: &#field_type) -> Result<(), &'static str> {
                                let sql = format!("UPDATE {} {} LIMIT 1", Self::get_table_name(), #update_all_where_sql);
                                match sqlx::query(&sql).bind(value_old).bind(value_new).execute(pool).await {
                                        Ok(_) => Ok(()),
                                        Err(err) => {
                                            println!("{}", err);
                                            Err("依据条件更新数据失败")
                                        }
                                }
                            }
                        });

                        // delete_by
                        let delete_by_method = Ident::new(
                            &format!("delete_by_{}", field_name),
                            proc_macro2::Span::call_site(),
                        );
                        let delete_where_sql = format!("WHERE {} = $1 LIMIT 1", &field_name);
                        tokens.push(quote!{
                            /// 依据字段 #current_field 删除单条记录
                            pub async fn #delete_by_method(pool: &common::types::Db, field_value: &#field_type) -> Result<(), &'static str> {
                                let sql = format!("DELETE FROM {} {}", Self::get_table_name(), #delete_where_sql);
                                match sqlx::query(&sql).bind(field_value).execute(pool).await {
                                        Ok(_) => Ok(()),
                                        Err(err) => {
                                            println!("{}", err);
                                            Err("依据条件删除数据失败")
                                        }
                                }
                            }
                        });

                        // delete_all_by
                        let delete_all_by_method = Ident::new(
                            &format!("delete_all_by_{}", field_name),
                            proc_macro2::Span::call_site(),
                        );
                        let delete_all_where_sql = format!("WHERE {} = $1", &field_name);
                        tokens.push(quote!{
                            /// 依据字段 #current_field 删除全部记录
                            pub async fn #delete_all_by_method(pool: &common::types::Db, field_value: &#field_type) -> Result<(), &'static str> {
                                let sql = format!("DELETE FROM {} {}", Self::get_table_name(), #delete_all_where_sql);
                                match sqlx::query(&sql).bind(field_value).execute(pool).await {
                                        Ok(_) => Ok(()),
                                        Err(err) => {
                                            println!("{}", err);
                                            Err("依据条件删除全部数据失败")
                                        }
                                }
                            }
                        });

                        // modify_by
                        let modify_by_method = Ident::new(
                            &format!("modify_by_{}", field_name),
                            proc_macro2::Span::call_site(),
                        );
                        let modify_where_sql = format!("SET {} = $1", &field_name);
                        tokens.push(quote!{
                            /// 依据字段 #current_field 更新单条记录
                            pub async fn #modify_by_method(&self, pool: &common::types::Db, field_value: &#field_type) -> Result<(), &'static str> {
                                let change_sql = format!("UPDATE {} {} WHERE id = {}", Self::get_table_name(), #modify_where_sql, &self.id);
                                println!("CHANGE SQL: {}", change_sql);
                                match sqlx::query(&change_sql).bind(field_value).execute(pool).await {
                                        Ok(_) => Ok(()),
                                        Err(err) => {
                                            println!("{}", err);
                                            Err("依据条件更新数据失败")
                                        }
                                }
                            }
                        });
                    }
                }
                _ => panic!("Expected named fields"),
            },
            _ => panic!("Expected a struct"),
        }
    }
    {
        let mut table_fields: Vec<String> = vec![];
        let DeriveInput { data, .. } = parse_macro_input!(input_clone);
        let mut create_set_fields: Vec<syn::Stmt> = vec![]; // 创建记录时的字段处理
        let mut create_builder_fields: Vec<syn::Stmt> = vec![]; // 创建记录时的 builder 处理
        let mut update_set_fields: Vec<syn::Stmt> = vec![]; // 更新记录时的字段处理
        let mut update_builder_fields: Vec<syn::Stmt> = vec![]; // 更新记录时的 builder 处理
        let mut updated_set_fields: Vec<syn::Stmt> = vec![];
        let mut updated_builder_fields: Vec<syn::Stmt> = vec![];

        match data {
            Data::Struct(s) => match s.fields {
                Fields::Named(f) => {
                    for field in f.named {
                        let field_name = field.ident.unwrap().to_string();
                        let field_type = field.ty.to_token_stream().to_string();

                        table_fields.push(field_name.to_owned());
                        if field_name == "id" {
                            continue;
                        }
                        // 创建记录 - created 字段
                        if field_name == "created" {
                            let created_field = format!(
                                r#"if self.created == 0 {{
                                fields.push("created".to_owned());
                                values.push(format!("${{}}", index));
                                index += 1;
                            }}"#
                            );
                            let created_stmt = syn::parse_str(&created_field).unwrap();
                            create_set_fields.push(created_stmt);

                            let val_set = format!(
                                r#"if self.created == 0 {{
                                builder = builder.bind(common::utils::dt::now_utc_micro());
                            }}"#
                            );
                            let if_set = syn::parse_str(&val_set).unwrap();
                            create_builder_fields.push(if_set);

                            continue;
                        }

                        // 创建记录 - updated 字段
                        if field_name == "updated" {
                            // 创建记录 - updated 字段
                            let created_field = format!(
                                r#"{{
                                fields.push("updated".to_owned());
                                values.push(format!("${{}}", index));
                                index += 1;
                            }}"#
                            );
                            let created_stmt = syn::parse_str(&created_field).unwrap();
                            create_set_fields.push(created_stmt);
                            // 创建记录 - updated builder
                            let created_builder_field = format!(
                                r#"{{
                                builder = builder.bind(common::utils::dt::now_utc_micro());
                            }}"#
                            );
                            let created_builder_stmt = syn::parse_str(&created_builder_field).unwrap();
                            create_builder_fields.push(created_builder_stmt);
                            // 修改记录 - updated 字段
                            let updated_field = format!(
                                r#"{{
                                values.push(format!("updated = ${{}}", index));
                                index += 1;
                            }}"#
                            );
                            let updated_stmt = syn::parse_str(&updated_field).unwrap();
                            updated_set_fields.push(updated_stmt);
                            // 修改记录 - updated builder
                            let updated_builder_field = format!(
                                r#"{{
                                builder = builder.bind(common::utils::dt::now_utc_micro());
                            }}"#
                            );
                            let updated_builder_stmt = syn::parse_str(&updated_builder_field).unwrap();
                            updated_builder_fields.push(updated_builder_stmt);

                            continue;
                        }

                        // 创建记录 - 依据条件
                        if field_type == "String" {
                            let create_field = format!(
                                r#"
                            if self.{} != "" {{
                                fields.push("{}".to_owned());
                                values.push(format!("${{}}", index));
                                index += 1;
                            }}
                            "#,
                                field_name, field_name
                            );
                            let create_field_stmt = syn::parse_str(&create_field).expect("解析判断默认字段的值失败");
                            create_set_fields.push(create_field_stmt);
                            // 创建记录 - builder
                            let create_builder = format!(
                                r#"if self.{} != "" {{
                                builder = builder.bind(&self.{});
                            }}"#,
                                field_name, field_name
                            );
                            let create_builder_stmt = syn::parse_str(&create_builder).unwrap();
                            create_builder_fields.push(create_builder_stmt);

                            // 修改记录 - 字符串字段
                            let update_field = format!(
                                r#"if self.{} != "" {{
                                values.push(format!("{} = ${{}}", index));
                                index += 1;
                            }}"#,
                                field_name, field_name
                            );
                            let update_field_stmt = syn::parse_str(&update_field).unwrap();
                            update_set_fields.push(update_field_stmt);
                            // 修改记录 - 字符串字段 builder
                            let update_builder = format!(
                                r#"if self.{} != "" {{
                                builder = builder.bind(&self.{});
                            }}"#,
                                field_name, field_name
                            );
                            let update_builder_stmt = syn::parse_str(&update_builder).unwrap();
                            update_builder_fields.push(update_builder_stmt);
                        } else if matches!(field_type.as_str(), "i8" | "i16" | "i32" | "i64") {
                            let create_field = format!(
                                r#"{{
                                fields.push("{}".to_owned());
                                values.push(format!("${{}}", index));
                                index += 1;
                            }}"#,
                                field_name
                            );
                            let create_field_stmt = syn::parse_str(&create_field)
                                .expect("解析判断默认字段的值数字失败");
                            create_set_fields.push(create_field_stmt);
                            // 创建记录 - builder
                            let create_builder = format!(
                                r#"
                                builder = builder.bind(&self.{});
                            "#,
                                field_name
                            );
                            let create_builder_stmt = syn::parse_str(&create_builder).unwrap();
                            create_builder_fields.push(create_builder_stmt);

                            // 修改记录 - 一般字段
                            let update_field = format!(
                                r#"{{
                                values.push(format!("{} = ${{}}", index));
                                index += 1;
                            }}"#,
                                field_name
                            );
                            let update_field_stmt = syn::parse_str(&update_field).unwrap();
                            update_set_fields.push(update_field_stmt);
                            // 修改记录 - 一般字段 builder
                            let update_builder = format!(
                                r#"
                                builder = builder.bind(&self.{});
                            "#,
                                field_name
                            );
                            let update_builder_stmt = syn::parse_str(&update_builder).unwrap();
                            update_builder_fields.push(update_builder_stmt);
                        }
                    }
                }
                _ => panic!("Expected named fields"),
            },
            _ => panic!("Expected a struct"),
        }
        let all_fields = table_fields.join(",");
        tokens.push(quote! {
            #[inline]
            pub fn get_fields() -> &'static str {
                #all_fields
            }
        });
        tokens.push(quote! {
            pub async fn create(&self, pool: &common::types::Db) -> Result<(), &'static str> {
                let mut insert_sql = String::from("INSERT INTO ");
                insert_sql.push_str(Self::get_table_name());
                insert_sql.push_str(" (");
                let mut fields: Vec<String> = vec![];
                let mut values: Vec<String> = vec![];
                let mut index = 1;
                #(#create_set_fields)*
                insert_sql.push_str(&fields.join(","));
                insert_sql.push_str(") VALUES (");
                insert_sql.push_str(&values.join(","));
                insert_sql.push_str(")");
                println!("INSERTING SQL: {}", &insert_sql);
                let mut builder = sqlx::query(&insert_sql);
                #(#create_builder_fields)*
                match builder.execute(pool).await {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        println!("create error: {}", e);
                        Err("记录创建失败")
                    }
                }
            }
        });
        tokens.push(quote!{
            pub async fn create_or_skip_by(&self, pool: &common::types::Db, skip_field: &'static str) -> Result<(), &'static str> {
                let mut insert_sql = String::from("INSERT INTO ");
                insert_sql.push_str(Self::get_table_name());
                insert_sql.push_str(" (");
                let mut fields: Vec<String> = vec![];
                let mut values: Vec<String> = vec![];
                let mut index = 1;
                #(#create_set_fields)*
                insert_sql.push_str(&fields.join(","));
                insert_sql.push_str(") VALUES (");
                insert_sql.push_str(&values.join(","));
                insert_sql.push_str(") ON CONFLICT (");
                insert_sql.push_str(skip_field);
                insert_sql.push_str(") DO NOTHING");
                println!("INSERTING OR SKIP SQL: {}", &insert_sql);
                let mut builder = sqlx::query(&insert_sql);
                #(#create_builder_fields)*
                match builder.execute(pool).await {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        println!("create or skip error: {}", e);
                        Err("记录创建失败")
                    }
                }
            }
        });
        tokens.push(quote!{
            /// 更新记录 - 修改指定字段
            pub async fn update(&self, pool: &common::types::Db, cond_fields: &[(&'static str, common::types::Val)]) -> Result<(), &'static str> {
                let mut update_sql = String::from("UPDATE ");
                update_sql.push_str(Self::get_table_name());
                update_sql.push_str(" SET ");
                let mut index = 1;
                let mut values: Vec<String> = vec![];
                for (field, _) in cond_fields {
                    values.push(format!("{} = ${}", field, index));
                    index += 1;
                }
                #(#updated_set_fields)*
                update_sql.push_str(&values.join(","));
                update_sql.push_str(&format!(" WHERE id = {}", self.id));
                println!("UPDATING SQL: {}", &update_sql);
                let mut builder = sqlx::query(&update_sql);
                for (_, val) in cond_fields {
                    match val {
                        common::types::Val::I8(rv)  => { builder = builder.bind::<i8>(*rv); },
                        common::types::Val::U8(rv)  => { builder = builder.bind::<i8>(*rv as i8); },
                        common::types::Val::I16(rv) => { builder = builder.bind::<i16>(*rv); },
                        common::types::Val::U16(rv) => { builder = builder.bind::<i16>(*rv as i16); },
                        common::types::Val::I32(rv) => { builder = builder.bind::<i32>(*rv); },
                        common::types::Val::U32(rv) => { builder = builder.bind::<i32>(*rv as i32); },
                        common::types::Val::I64(rv) => { builder = builder.bind::<i64>(*rv); },
                        common::types::Val::U64(rv) => { builder = builder.bind::<i64>(*rv as i64); },
                        common::types::Val::F32(rv) => { builder = builder.bind::<f32>(*rv); },
                        common::types::Val::F64(rv) => { builder = builder.bind::<f64>(*rv); },
                        common::types::Val::Str(rv) => { builder = builder.bind(rv); },
                        common::types::Val::S(rv)   => { builder = builder.bind(rv); },
                        common::types::Val::Bool(rv) => { builder = builder.bind(rv); },
                        _ => { continue; }
                    }
                }
                #(#updated_builder_fields)*
                match builder.execute(pool).await {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        println!("Update error: {}", e);
                        Err("记录修改失败")
                    }
                }
            }
        });
        tokens.push(quote! {
            /// 更新记录 - 修改所有字段
            pub async fn save(&self, pool: &common::types::Db) -> Result<(), &'static str> {
                let mut save_sql = String::from("UPDATE ");
                save_sql.push_str(Self::get_table_name());
                save_sql.push_str(" SET ");
                let mut index = 1;
                let mut values: Vec<String> = vec![];
                #(#update_set_fields)*
                #(#updated_set_fields)*
                save_sql.push_str(&values.join(","));
                save_sql.push_str(&format!(" WHERE id = {}", self.id));
                println!("SAVING SQL: {}", &save_sql);
                let mut builder = sqlx::query(&save_sql);
                #(#update_builder_fields)*
                #(#updated_builder_fields)*
                match builder.execute(pool).await {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        println!("Update error: {}", e);
                        Err("记录修改失败")
                    }
                }
            }
        });
        tokens.push(quote! {
            /// 删除记录
            pub async fn delete(&self, pool: &common::types::Db) -> Result<(), &'static str> {
                let mut delete_sql = String::from("DELETE FROM ");
                delete_sql.push_str(Self::get_table_name());
                delete_sql.push_str(&format!(" WHERE id = {}", self.id));
                println!("DELETING SQL: {}", &delete_sql);
                match sqlx::query(&delete_sql).execute(pool).await {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        println!("Delete error: {}", e);
                        Err("记录删除失败")
                    }
                }
            }
        });
    }

    // 生成代码
    let DeriveInput { ident, .. } = parse_macro_input!(input_clone2);
    let expanded = quote! {
        impl #ident {
            #(#tokens)*
        }
    };

    expanded.into() // 将代码转换为 token 流并返回
}
