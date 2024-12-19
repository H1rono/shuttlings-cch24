use std::{borrow::Cow, future::Future, sync::Arc};

use crate::{bucket, cookie, handlers::auth_token, jwt};

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Builder<
    SeekUrl = (),
    ManifestKeyword = (),
    MilkBucket = (),
    JwtManager = (),
    CookieManager = (),
    JwtDecoder = (),
> {
    seek_url: SeekUrl,
    manifest_keyword: ManifestKeyword,
    milk_bucket: MilkBucket,
    jwt_manager: JwtManager,
    cookie_manager: CookieManager,
    jwt_decoder: JwtDecoder,
}

impl Builder {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<SeekUrl, ManifestKeyword, MilkBucket, JwtManager, CookieManager, JwtDecoder>
    Builder<SeekUrl, ManifestKeyword, MilkBucket, JwtManager, CookieManager, JwtDecoder>
{
    pub fn seek_url<'s, S>(
        self,
        value: S,
    ) -> Builder<String, ManifestKeyword, MilkBucket, JwtManager, CookieManager, JwtDecoder>
    where
        S: Into<Cow<'s, str>>,
    {
        let Self {
            manifest_keyword,
            milk_bucket,
            jwt_manager,
            cookie_manager,
            jwt_decoder,
            ..
        } = self;
        let seek_url = value.into().into_owned();
        Builder {
            seek_url,
            manifest_keyword,
            milk_bucket,
            jwt_manager,
            cookie_manager,
            jwt_decoder,
        }
    }

    pub fn manifest_keyword<'s, S>(
        self,
        value: S,
    ) -> Builder<SeekUrl, String, MilkBucket, JwtManager, CookieManager, JwtDecoder>
    where
        S: Into<Cow<'s, str>>,
    {
        let Self {
            seek_url,
            milk_bucket,
            jwt_manager,
            cookie_manager,
            jwt_decoder,
            ..
        } = self;
        let manifest_keyword = value.into().into_owned();
        Builder {
            seek_url,
            manifest_keyword,
            milk_bucket,
            jwt_manager,
            cookie_manager,
            jwt_decoder,
        }
    }

    pub fn milk_bucket(
        self,
        value: bucket::MilkBucket,
    ) -> Builder<SeekUrl, ManifestKeyword, bucket::MilkBucket, JwtManager, CookieManager, JwtDecoder>
    {
        let Self {
            seek_url,
            manifest_keyword,
            jwt_manager,
            cookie_manager,
            jwt_decoder,
            ..
        } = self;
        Builder {
            seek_url,
            manifest_keyword,
            milk_bucket: value,
            jwt_manager,
            cookie_manager,
            jwt_decoder,
        }
    }

    pub fn jwt_manager(
        self,
        value: jwt::Manager,
    ) -> Builder<SeekUrl, ManifestKeyword, MilkBucket, jwt::Manager, CookieManager, JwtDecoder>
    {
        let Self {
            seek_url,
            manifest_keyword,
            milk_bucket,
            cookie_manager,
            jwt_decoder,
            ..
        } = self;
        Builder {
            seek_url,
            manifest_keyword,
            milk_bucket,
            jwt_manager: value,
            cookie_manager,
            jwt_decoder,
        }
    }

    pub fn cookie_manager(
        self,
        value: cookie::Manager,
    ) -> Builder<SeekUrl, ManifestKeyword, MilkBucket, JwtManager, cookie::Manager, JwtDecoder>
    {
        let Self {
            seek_url,
            manifest_keyword,
            milk_bucket,
            jwt_manager,
            jwt_decoder,
            ..
        } = self;
        Builder {
            seek_url,
            manifest_keyword,
            milk_bucket,
            jwt_manager,
            cookie_manager: value,
            jwt_decoder,
        }
    }

    pub fn jwt_decoder(
        self,
        value: jwt::Decoder,
    ) -> Builder<SeekUrl, ManifestKeyword, MilkBucket, JwtManager, CookieManager, jwt::Decoder>
    {
        let Self {
            seek_url,
            manifest_keyword,
            milk_bucket,
            jwt_manager,
            cookie_manager,
            ..
        } = self;
        Builder {
            seek_url,
            manifest_keyword,
            milk_bucket,
            jwt_manager,
            cookie_manager,
            jwt_decoder: value,
        }
    }
}

impl
    Builder<
        String,
        String,
        bucket::MilkBucket,
        crate::jwt::Manager,
        crate::cookie::Manager,
        crate::jwt::Decoder,
    >
{
    pub fn build(self) -> super::State {
        use crate::handlers::{manifest, milk, seek};

        let Self {
            seek_url,
            manifest_keyword,
            milk_bucket,
            jwt_manager,
            cookie_manager,
            jwt_decoder,
        } = self;
        let seek_state = seek::State::builder().seek_url(seek_url).build();
        let manifest_state = manifest::State::builder()
            .manifest_keyword(manifest_keyword)
            .build();
        let milk = milk::State::builder().bucket(milk_bucket).build();
        let auth_token = auth_token::State::builder()
            .jwt_manager(jwt_manager)
            .cookie_manager(cookie_manager)
            .decoder(jwt_decoder)
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
