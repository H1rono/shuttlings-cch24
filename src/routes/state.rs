use std::{borrow::Cow, future::Future, sync::Arc};

use crate::{cookie, handlers::auth_token, jwt};

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Builder<
    SeekUrl = (),
    ManifestKeyword = (),
    MilkFull = (),
    MilkInitial = (),
    JwtManager = (),
    CookieManager = (),
> {
    seek_url: SeekUrl,
    manifest_keyword: ManifestKeyword,
    milk_full: MilkFull,
    milk_initial: MilkInitial,
    jwt_manager: JwtManager,
    cookie_manager: CookieManager,
}

impl Builder {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<SeekUrl, ManifestKeyword, MilkFull, MilkInitial, JwtManager, CookieManager>
    Builder<SeekUrl, ManifestKeyword, MilkFull, MilkInitial, JwtManager, CookieManager>
{
    pub fn seek_url<'s, S>(
        self,
        value: S,
    ) -> Builder<String, ManifestKeyword, MilkFull, MilkInitial, JwtManager, CookieManager>
    where
        S: Into<Cow<'s, str>>,
    {
        let Self {
            manifest_keyword,
            milk_full,
            milk_initial,
            jwt_manager,
            cookie_manager,
            ..
        } = self;
        let seek_url = value.into().into_owned();
        Builder {
            seek_url,
            manifest_keyword,
            milk_full,
            milk_initial,
            jwt_manager,
            cookie_manager,
        }
    }

    pub fn manifest_keyword<'s, S>(
        self,
        value: S,
    ) -> Builder<SeekUrl, String, MilkFull, MilkInitial, JwtManager, CookieManager>
    where
        S: Into<Cow<'s, str>>,
    {
        let Self {
            seek_url,
            milk_full,
            milk_initial,
            jwt_manager,
            cookie_manager,
            ..
        } = self;
        let manifest_keyword = value.into().into_owned();
        Builder {
            seek_url,
            manifest_keyword,
            milk_full,
            milk_initial,
            jwt_manager,
            cookie_manager,
        }
    }

    pub fn milk_full(
        self,
        value: f32,
    ) -> Builder<SeekUrl, ManifestKeyword, f32, MilkInitial, JwtManager, CookieManager> {
        let Self {
            seek_url,
            manifest_keyword,
            milk_initial,
            jwt_manager,
            cookie_manager,
            ..
        } = self;
        Builder {
            seek_url,
            manifest_keyword,
            milk_full: value,
            milk_initial,
            jwt_manager,
            cookie_manager,
        }
    }

    pub fn milk_initial(
        self,
        value: f32,
    ) -> Builder<SeekUrl, ManifestKeyword, MilkFull, f32, JwtManager, CookieManager> {
        let Self {
            seek_url,
            manifest_keyword,
            milk_full,
            jwt_manager,
            cookie_manager,
            ..
        } = self;
        Builder {
            seek_url,
            manifest_keyword,
            milk_full,
            milk_initial: value,
            jwt_manager,
            cookie_manager,
        }
    }

    pub fn jwt_manager(
        self,
        value: jwt::Manager,
    ) -> Builder<SeekUrl, ManifestKeyword, MilkFull, MilkInitial, jwt::Manager, CookieManager> {
        let Self {
            seek_url,
            manifest_keyword,
            milk_full,
            milk_initial,
            cookie_manager,
            ..
        } = self;
        Builder {
            seek_url,
            manifest_keyword,
            milk_full,
            milk_initial,
            jwt_manager: value,
            cookie_manager,
        }
    }

    pub fn cookie_manager(
        self,
        value: cookie::Manager,
    ) -> Builder<SeekUrl, ManifestKeyword, MilkFull, MilkInitial, JwtManager, cookie::Manager> {
        let Self {
            seek_url,
            manifest_keyword,
            milk_full,
            milk_initial,
            jwt_manager,
            ..
        } = self;
        Builder {
            seek_url,
            manifest_keyword,
            milk_full,
            milk_initial,
            jwt_manager,
            cookie_manager: value,
        }
    }
}

impl Builder<String, String, f32, f32, crate::jwt::Manager, crate::cookie::Manager> {
    pub fn build(self) -> super::State {
        use crate::handlers::{manifest, milk, seek};

        let Self {
            seek_url,
            manifest_keyword,
            milk_full,
            milk_initial,
            jwt_manager,
            cookie_manager,
        } = self;
        let seek_state = seek::State::builder().seek_url(seek_url).build();
        let manifest_state = manifest::State::builder()
            .manifest_keyword(manifest_keyword)
            .build();
        let milk = milk::State::builder()
            .full(milk_full)
            .initial(milk_initial)
            .build();
        let auth_token = auth_token::State::builder()
            .jwt_manager(jwt_manager)
            .cookie_manager(cookie_manager)
            .build();
        super::State {
            seek: Arc::new(seek_state),
            manifest: Arc::new(manifest_state),
            milk: Arc::new(milk),
            connect4: Arc::new(Default::default()),
            auth_token: Arc::new(auth_token),
        }
    }
}

impl super::State {
    pub fn builder() -> Builder {
        Builder::new()
    }

    pub fn bg_task(&self) -> impl Future<Output = ()> + Send + 'static {
        use crate::bucket::{milk, Liters};

        // FIXME: expose configuration
        let rate = milk::RefillRate::per_sec(Liters(1.0));
        self.milk.refill_task(rate)
    }
}
