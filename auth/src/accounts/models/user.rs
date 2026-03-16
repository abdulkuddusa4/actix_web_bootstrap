#![allow(warnings)]

use sea_orm::{ActiveValue, entity::prelude::*};
use sha2::{Sha512, Digest};


#[derive(Clone, Debug, PartialEq, Eq, DeriveActiveEnum, EnumIter)]
#[sea_orm(db_type="Integer", rs_type="i32")]
enum OtpReason{
    #[sea_orm(num_value=0)]
    Login,
    #[sea_orm(num_value=1)]
    PasswordReset,
    #[sea_orm(num_value=2)]
    EmailVerification,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Email(String);


impl Email{
    pub fn parse(st: &str)->Option<Self>{
        let parts = st.split("@").collect::<Vec<&str>>();

        if parts.len() != 2{
            return None;
        }
        if parts[1].split(".").collect::<Vec<&str>>().len() < 2{
            return None;
        }

        return Some(Self(st.to_owned()))
    }

    pub fn as_ref(&self)->&str{
        &self.0
    }
}


impl Default for OtpReason{
    fn default() -> Self {
        OtpReason::EmailVerification
    }
}


#[sea_orm::model]
#[derive(Clone, Default, Debug, DeriveEntityModel, PartialEq, Eq)]
#[sea_orm(table_name = "users")]
pub struct Model{
    #[sea_orm(primary_key)]
    id: i32,

    #[sea_orm(unique)]
    email: String,

    otp_reason: OtpReason,
    otp_expiry: DateTimeUtc,
    digest: String,
    salt: String,
}

impl ActiveModelBehavior for ActiveModel{}


impl ActiveModel{
    pub async fn new_user(db: &sea_orm::DatabaseConnection, email: &str, password: &str){
        let user = Self{email: sea_orm::ActiveValue::set(email.to_owned()),..Default::default()};
    }

    pub async fn set_password(&mut self, new_password: &str){
        let eml: &str = self.email.try_as_ref().unwrap();
        let salt: String = format!("{}|{}", eml, crate::utils::generate_random_string(10));
        let mut hasher = Sha512::new();
        hasher.update(&salt);
        hasher.update(new_password);
        let digest = format!("{:x}", hasher.finalize());

        self.salt = ActiveValue::Set(salt);
        self.digest = sea_orm::ActiveValue::Set(digest);
    }
}


impl Model{
    pub async fn check_password(&self, password: &str)->bool{
        let mut hasher = Sha512::new();
        hasher.update(&self.salt);
        hasher.update(&self.digest);
        self.digest == format!("{:x}", hasher.finalize())
    }
}