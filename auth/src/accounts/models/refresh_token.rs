use std::{any::Any, collections::HashMap};

use sea_orm::entity::prelude::*;

use super::user::Model as UserModel;


#[sea_orm::model]
#[derive(Clone, Default, Debug, DeriveEntityModel, PartialEq, Eq)]
#[sea_orm(table_name="refresh_tokens")]
pub struct Model{
    #[sea_orm(primary_key)]
    id: i32,
    user_id: i32,
    #[sea_orm(indexed)]
    refresh_token: String
}

impl ActiveModelBehavior for ActiveModel{}



impl ActiveModel{
    async fn new(payload: HashMap<String, String>){

    }
}