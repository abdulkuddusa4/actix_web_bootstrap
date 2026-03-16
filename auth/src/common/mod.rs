#[derive(Clone, Debug, PartialEq, Eq)]

pub struct Email(String);

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