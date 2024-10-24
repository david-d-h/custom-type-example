use std::io::Write;
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};

use rand::distributions::Uniform;
use rand::prelude::Distribution;
use rand::RngCore;

use diesel::{
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    query_builder::QueryId,
    serialize::{IsNull, ToSql},
    sql_types::SqlType,
};

use once_cell::sync::Lazy;

#[derive(Debug, Clone, Copy, Default, QueryId, SqlType)]
#[diesel(postgres_type(name = "passcode", schema = "users"))]
pub struct CodeType;

#[repr(transparent)]
#[derive(Debug, Copy, Clone, AsExpression, FromSqlRow)]
#[diesel(sql_type = CodeType)]
pub struct Code([u8; Self::LEN]);

pub static RANGE: Lazy<Uniform<u8>> = Lazy::new(|| Uniform::new_inclusive(b'0', b'9'));

pub fn generate<R: RngCore + ?Sized>(rng: &mut R) -> Code {
    let mut code = Code(const { [0; Code::LEN] });
    code.iter_mut().for_each(|n| *n = RANGE.sample(rng));
    code
}

impl Code {
    pub const LEN: usize = 24;

    #[inline]
    pub fn generate<R: RngCore + ?Sized>(rng: &mut R) -> Self {
        generate(rng)
    }

    #[inline]
    pub const fn into_inner(self) -> [u8; Self::LEN] {
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn as_chars(&self) -> [char; Self::LEN] {
        let mut chars = [const { MaybeUninit::uninit() }; Self::LEN];

        for (i, c) in chars.iter_mut().enumerate() {
            _ = c.write(self.0[i] as char);
        }

        unsafe { std::mem::transmute(chars) }
    }
}

impl ToString for Code {
    #[inline]
    fn to_string(&self) -> String {
        String::from_iter(self.as_chars())
    }
}

impl Deref for Code {
    type Target = [u8; Self::LEN];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Code {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromSql<CodeType, Pg> for Code {
    #[inline]
    fn from_sql(
        bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        let bytes = bytes.as_bytes();

        assert_eq!(bytes.len(), Self::LEN);

        assert!(bytes.iter().all(u8::is_ascii_digit));

        Ok(unsafe {
            // SAFETY: the length of the slice was checked.
            let bytes = &*(bytes.as_ptr() as *const [u8; Self::LEN]);
            Code(*bytes)
        })
    }
}

impl ToSql<CodeType, Pg> for Code {
    #[inline]
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        out.write_all(&self.0)?;
        Ok(IsNull::No)
    }
}
