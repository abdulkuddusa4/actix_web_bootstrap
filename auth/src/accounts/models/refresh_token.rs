use sea_orm::entity::prelude::*;

use super::user::Model as UserModel;


#[sea_orm::model]
#[derive(Clone, Default, Debug, DeriveEntityModel, PartialEq, Eq)]
#[sea_orm(table_name="refresh_tokens")]
pub struct Model{
    #[sea_orm(primary_key)]
    id: i32
}

impl ActiveModelBehavior for ActiveModel{}



impl ActiveModel{
    
}