#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
pub mod model {
    use std::error::Error;
    use rkyv::de::deserializers::SharedDeserializeMap;
    use rkyv::{Archive, Deserialize, Serialize};
    use zerocopy::AsBytes;
    pub mod article {
        use rkyv::{Archive, Deserialize, Serialize};
        use super::shop::Shop;
        use super::Identifiable;
        #[archive_attr(derive(bytecheck::CheckBytes, Debug))]
        pub struct Article {
            pub id: <Article as Identifiable>::Id,
            pub name: String,
            pub description: Option<String>,
            pub image_id: Option<u32>,
            pub shops: Option<Vec<<Shop as Identifiable>::Id>>,
        }
        #[automatically_derived]
        ///An archived [`Article`]
        #[repr()]
        pub struct ArchivedArticle
        where
            <Article as Identifiable>::Id: ::rkyv::Archive,
            String: ::rkyv::Archive,
            Option<String>: ::rkyv::Archive,
            Option<u32>: ::rkyv::Archive,
            Option<Vec<<Shop as Identifiable>::Id>>: ::rkyv::Archive,
        {
            ///The archived counterpart of [`Article::id`]
            pub id: ::rkyv::Archived<<Article as Identifiable>::Id>,
            ///The archived counterpart of [`Article::name`]
            pub name: ::rkyv::Archived<String>,
            ///The archived counterpart of [`Article::description`]
            pub description: ::rkyv::Archived<Option<String>>,
            ///The archived counterpart of [`Article::image_id`]
            pub image_id: ::rkyv::Archived<Option<u32>>,
            ///The archived counterpart of [`Article::shops`]
            pub shops: ::rkyv::Archived<Option<Vec<<Shop as Identifiable>::Id>>>,
        }
        const _: () = {
            use ::core::{convert::Infallible, marker::PhantomData};
            use bytecheck::{
                CheckBytes, EnumCheckError, ErrorBox, StructCheckError,
                TupleStructCheckError,
            };
            impl<__C: ?Sized> CheckBytes<__C> for ArchivedArticle
            where
                <Article as Identifiable>::Id: ::rkyv::Archive,
                String: ::rkyv::Archive,
                Option<String>: ::rkyv::Archive,
                Option<u32>: ::rkyv::Archive,
                Option<Vec<<Shop as Identifiable>::Id>>: ::rkyv::Archive,
                ::rkyv::Archived<<Article as Identifiable>::Id>: CheckBytes<__C>,
                ::rkyv::Archived<String>: CheckBytes<__C>,
                ::rkyv::Archived<Option<String>>: CheckBytes<__C>,
                ::rkyv::Archived<Option<u32>>: CheckBytes<__C>,
                ::rkyv::Archived<
                    Option<Vec<<Shop as Identifiable>::Id>>,
                >: CheckBytes<__C>,
            {
                type Error = StructCheckError;
                unsafe fn check_bytes<'__bytecheck>(
                    value: *const Self,
                    context: &mut __C,
                ) -> Result<&'__bytecheck Self, Self::Error> {
                    let bytes = value.cast::<u8>();
                    <::rkyv::Archived<
                        <Article as Identifiable>::Id,
                    > as CheckBytes<__C>>::check_bytes(&raw const (*value).id, context)
                        .map_err(|e| StructCheckError {
                            field_name: "id",
                            inner: ErrorBox::new(e),
                        })?;
                    <::rkyv::Archived<
                        String,
                    > as CheckBytes<__C>>::check_bytes(&raw const (*value).name, context)
                        .map_err(|e| StructCheckError {
                            field_name: "name",
                            inner: ErrorBox::new(e),
                        })?;
                    <::rkyv::Archived<
                        Option<String>,
                    > as CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).description, context)
                        .map_err(|e| StructCheckError {
                            field_name: "description",
                            inner: ErrorBox::new(e),
                        })?;
                    <::rkyv::Archived<
                        Option<u32>,
                    > as CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).image_id, context)
                        .map_err(|e| StructCheckError {
                            field_name: "image_id",
                            inner: ErrorBox::new(e),
                        })?;
                    <::rkyv::Archived<
                        Option<Vec<<Shop as Identifiable>::Id>>,
                    > as CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).shops, context)
                        .map_err(|e| StructCheckError {
                            field_name: "shops",
                            inner: ErrorBox::new(e),
                        })?;
                    Ok(&*value)
                }
            }
        };
        #[automatically_derived]
        impl ::core::fmt::Debug for ArchivedArticle
        where
            <Article as Identifiable>::Id: ::rkyv::Archive,
            String: ::rkyv::Archive,
            Option<String>: ::rkyv::Archive,
            Option<u32>: ::rkyv::Archive,
            Option<Vec<<Shop as Identifiable>::Id>>: ::rkyv::Archive,
        {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field5_finish(
                    f,
                    "ArchivedArticle",
                    "id",
                    &&self.id,
                    "name",
                    &&self.name,
                    "description",
                    &&self.description,
                    "image_id",
                    &&self.image_id,
                    "shops",
                    &&self.shops,
                )
            }
        }
        #[automatically_derived]
        ///The resolver for an archived [`Article`]
        pub struct ArticleResolver
        where
            <Article as Identifiable>::Id: ::rkyv::Archive,
            String: ::rkyv::Archive,
            Option<String>: ::rkyv::Archive,
            Option<u32>: ::rkyv::Archive,
            Option<Vec<<Shop as Identifiable>::Id>>: ::rkyv::Archive,
        {
            id: ::rkyv::Resolver<<Article as Identifiable>::Id>,
            name: ::rkyv::Resolver<String>,
            description: ::rkyv::Resolver<Option<String>>,
            image_id: ::rkyv::Resolver<Option<u32>>,
            shops: ::rkyv::Resolver<Option<Vec<<Shop as Identifiable>::Id>>>,
        }
        #[automatically_derived]
        const _: () = {
            use ::core::marker::PhantomData;
            use ::rkyv::{out_field, Archive, Archived};
            impl Archive for Article
            where
                <Article as Identifiable>::Id: ::rkyv::Archive,
                String: ::rkyv::Archive,
                Option<String>: ::rkyv::Archive,
                Option<u32>: ::rkyv::Archive,
                Option<Vec<<Shop as Identifiable>::Id>>: ::rkyv::Archive,
            {
                type Archived = ArchivedArticle;
                type Resolver = ArticleResolver;
                #[allow(clippy::unit_arg)]
                #[inline]
                unsafe fn resolve(
                    &self,
                    pos: usize,
                    resolver: Self::Resolver,
                    out: *mut Self::Archived,
                ) {
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).id;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve((&self.id), pos + fp, resolver.id, fo);
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).name;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve((&self.name), pos + fp, resolver.name, fo);
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).description;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve(
                        (&self.description),
                        pos + fp,
                        resolver.description,
                        fo,
                    );
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).image_id;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve(
                        (&self.image_id),
                        pos + fp,
                        resolver.image_id,
                        fo,
                    );
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).shops;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve(
                        (&self.shops),
                        pos + fp,
                        resolver.shops,
                        fo,
                    );
                }
            }
        };
        #[automatically_derived]
        const _: () = {
            use ::rkyv::{Archive, Fallible, Serialize};
            impl<__S: Fallible + ?Sized> Serialize<__S> for Article
            where
                <Article as Identifiable>::Id: Serialize<__S>,
                String: Serialize<__S>,
                Option<String>: Serialize<__S>,
                Option<u32>: Serialize<__S>,
                Option<Vec<<Shop as Identifiable>::Id>>: Serialize<__S>,
            {
                #[inline]
                fn serialize(
                    &self,
                    serializer: &mut __S,
                ) -> ::core::result::Result<Self::Resolver, __S::Error> {
                    Ok(ArticleResolver {
                        id: Serialize::<__S>::serialize(&self.id, serializer)?,
                        name: Serialize::<__S>::serialize(&self.name, serializer)?,
                        description: Serialize::<
                            __S,
                        >::serialize(&self.description, serializer)?,
                        image_id: Serialize::<
                            __S,
                        >::serialize(&self.image_id, serializer)?,
                        shops: Serialize::<__S>::serialize(&self.shops, serializer)?,
                    })
                }
            }
        };
        #[automatically_derived]
        const _: () = {
            use ::rkyv::{Archive, Archived, Deserialize, Fallible};
            impl<__D: Fallible + ?Sized> Deserialize<Article, __D> for Archived<Article>
            where
                <Article as Identifiable>::Id: Archive,
                Archived<
                    <Article as Identifiable>::Id,
                >: Deserialize<<Article as Identifiable>::Id, __D>,
                String: Archive,
                Archived<String>: Deserialize<String, __D>,
                Option<String>: Archive,
                Archived<Option<String>>: Deserialize<Option<String>, __D>,
                Option<u32>: Archive,
                Archived<Option<u32>>: Deserialize<Option<u32>, __D>,
                Option<Vec<<Shop as Identifiable>::Id>>: Archive,
                Archived<
                    Option<Vec<<Shop as Identifiable>::Id>>,
                >: Deserialize<Option<Vec<<Shop as Identifiable>::Id>>, __D>,
            {
                #[inline]
                fn deserialize(
                    &self,
                    deserializer: &mut __D,
                ) -> ::core::result::Result<Article, __D::Error> {
                    Ok(Article {
                        id: Deserialize::<
                            <Article as Identifiable>::Id,
                            __D,
                        >::deserialize(&self.id, deserializer)?,
                        name: Deserialize::<
                            String,
                            __D,
                        >::deserialize(&self.name, deserializer)?,
                        description: Deserialize::<
                            Option<String>,
                            __D,
                        >::deserialize(&self.description, deserializer)?,
                        image_id: Deserialize::<
                            Option<u32>,
                            __D,
                        >::deserialize(&self.image_id, deserializer)?,
                        shops: Deserialize::<
                            Option<Vec<<Shop as Identifiable>::Id>>,
                            __D,
                        >::deserialize(&self.shops, deserializer)?,
                    })
                }
            }
        };
        #[automatically_derived]
        impl ::core::fmt::Debug for Article {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field5_finish(
                    f,
                    "Article",
                    "id",
                    &&self.id,
                    "name",
                    &&self.name,
                    "description",
                    &&self.description,
                    "image_id",
                    &&self.image_id,
                    "shops",
                    &&self.shops,
                )
            }
        }
        impl Identifiable for Article {
            type Id = u64;
        }
    }
    pub mod item {
        use rkyv::{Archive, Deserialize, Serialize};
        use super::article::Article;
        use super::Identifiable;
        #[archive_attr(derive(bytecheck::CheckBytes))]
        pub struct Item {
            pub id: <Item as Identifiable>::Id,
            pub checked: bool,
            pub name: String,
            pub amount: Option<u64>,
            pub unit: Option<Unit>,
            pub article_id: Option<<Article as Identifiable>::Id>,
            pub alternative_article_ids: Option<Vec<<Article as Identifiable>::Id>>,
        }
        #[automatically_derived]
        ///An archived [`Item`]
        #[repr()]
        pub struct ArchivedItem
        where
            <Item as Identifiable>::Id: ::rkyv::Archive,
            bool: ::rkyv::Archive,
            String: ::rkyv::Archive,
            Option<u64>: ::rkyv::Archive,
            Option<Unit>: ::rkyv::Archive,
            Option<<Article as Identifiable>::Id>: ::rkyv::Archive,
            Option<Vec<<Article as Identifiable>::Id>>: ::rkyv::Archive,
        {
            ///The archived counterpart of [`Item::id`]
            pub id: ::rkyv::Archived<<Item as Identifiable>::Id>,
            ///The archived counterpart of [`Item::checked`]
            pub checked: ::rkyv::Archived<bool>,
            ///The archived counterpart of [`Item::name`]
            pub name: ::rkyv::Archived<String>,
            ///The archived counterpart of [`Item::amount`]
            pub amount: ::rkyv::Archived<Option<u64>>,
            ///The archived counterpart of [`Item::unit`]
            pub unit: ::rkyv::Archived<Option<Unit>>,
            ///The archived counterpart of [`Item::article_id`]
            pub article_id: ::rkyv::Archived<Option<<Article as Identifiable>::Id>>,
            ///The archived counterpart of [`Item::alternative_article_ids`]
            pub alternative_article_ids: ::rkyv::Archived<
                Option<Vec<<Article as Identifiable>::Id>>,
            >,
        }
        const _: () = {
            use ::core::{convert::Infallible, marker::PhantomData};
            use bytecheck::{
                CheckBytes, EnumCheckError, ErrorBox, StructCheckError,
                TupleStructCheckError,
            };
            impl<__C: ?Sized> CheckBytes<__C> for ArchivedItem
            where
                <Item as Identifiable>::Id: ::rkyv::Archive,
                bool: ::rkyv::Archive,
                String: ::rkyv::Archive,
                Option<u64>: ::rkyv::Archive,
                Option<Unit>: ::rkyv::Archive,
                Option<<Article as Identifiable>::Id>: ::rkyv::Archive,
                Option<Vec<<Article as Identifiable>::Id>>: ::rkyv::Archive,
                ::rkyv::Archived<<Item as Identifiable>::Id>: CheckBytes<__C>,
                ::rkyv::Archived<bool>: CheckBytes<__C>,
                ::rkyv::Archived<String>: CheckBytes<__C>,
                ::rkyv::Archived<Option<u64>>: CheckBytes<__C>,
                ::rkyv::Archived<Option<Unit>>: CheckBytes<__C>,
                ::rkyv::Archived<Option<<Article as Identifiable>::Id>>: CheckBytes<__C>,
                ::rkyv::Archived<
                    Option<Vec<<Article as Identifiable>::Id>>,
                >: CheckBytes<__C>,
            {
                type Error = StructCheckError;
                unsafe fn check_bytes<'__bytecheck>(
                    value: *const Self,
                    context: &mut __C,
                ) -> Result<&'__bytecheck Self, Self::Error> {
                    let bytes = value.cast::<u8>();
                    <::rkyv::Archived<
                        <Item as Identifiable>::Id,
                    > as CheckBytes<__C>>::check_bytes(&raw const (*value).id, context)
                        .map_err(|e| StructCheckError {
                            field_name: "id",
                            inner: ErrorBox::new(e),
                        })?;
                    <::rkyv::Archived<
                        bool,
                    > as CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).checked, context)
                        .map_err(|e| StructCheckError {
                            field_name: "checked",
                            inner: ErrorBox::new(e),
                        })?;
                    <::rkyv::Archived<
                        String,
                    > as CheckBytes<__C>>::check_bytes(&raw const (*value).name, context)
                        .map_err(|e| StructCheckError {
                            field_name: "name",
                            inner: ErrorBox::new(e),
                        })?;
                    <::rkyv::Archived<
                        Option<u64>,
                    > as CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).amount, context)
                        .map_err(|e| StructCheckError {
                            field_name: "amount",
                            inner: ErrorBox::new(e),
                        })?;
                    <::rkyv::Archived<
                        Option<Unit>,
                    > as CheckBytes<__C>>::check_bytes(&raw const (*value).unit, context)
                        .map_err(|e| StructCheckError {
                            field_name: "unit",
                            inner: ErrorBox::new(e),
                        })?;
                    <::rkyv::Archived<
                        Option<<Article as Identifiable>::Id>,
                    > as CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).article_id, context)
                        .map_err(|e| StructCheckError {
                            field_name: "article_id",
                            inner: ErrorBox::new(e),
                        })?;
                    <::rkyv::Archived<
                        Option<Vec<<Article as Identifiable>::Id>>,
                    > as CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).alternative_article_ids, context)
                        .map_err(|e| StructCheckError {
                            field_name: "alternative_article_ids",
                            inner: ErrorBox::new(e),
                        })?;
                    Ok(&*value)
                }
            }
        };
        #[automatically_derived]
        ///The resolver for an archived [`Item`]
        pub struct ItemResolver
        where
            <Item as Identifiable>::Id: ::rkyv::Archive,
            bool: ::rkyv::Archive,
            String: ::rkyv::Archive,
            Option<u64>: ::rkyv::Archive,
            Option<Unit>: ::rkyv::Archive,
            Option<<Article as Identifiable>::Id>: ::rkyv::Archive,
            Option<Vec<<Article as Identifiable>::Id>>: ::rkyv::Archive,
        {
            id: ::rkyv::Resolver<<Item as Identifiable>::Id>,
            checked: ::rkyv::Resolver<bool>,
            name: ::rkyv::Resolver<String>,
            amount: ::rkyv::Resolver<Option<u64>>,
            unit: ::rkyv::Resolver<Option<Unit>>,
            article_id: ::rkyv::Resolver<Option<<Article as Identifiable>::Id>>,
            alternative_article_ids: ::rkyv::Resolver<
                Option<Vec<<Article as Identifiable>::Id>>,
            >,
        }
        #[automatically_derived]
        const _: () = {
            use ::core::marker::PhantomData;
            use ::rkyv::{out_field, Archive, Archived};
            impl Archive for Item
            where
                <Item as Identifiable>::Id: ::rkyv::Archive,
                bool: ::rkyv::Archive,
                String: ::rkyv::Archive,
                Option<u64>: ::rkyv::Archive,
                Option<Unit>: ::rkyv::Archive,
                Option<<Article as Identifiable>::Id>: ::rkyv::Archive,
                Option<Vec<<Article as Identifiable>::Id>>: ::rkyv::Archive,
            {
                type Archived = ArchivedItem;
                type Resolver = ItemResolver;
                #[allow(clippy::unit_arg)]
                #[inline]
                unsafe fn resolve(
                    &self,
                    pos: usize,
                    resolver: Self::Resolver,
                    out: *mut Self::Archived,
                ) {
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).id;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve((&self.id), pos + fp, resolver.id, fo);
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).checked;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve(
                        (&self.checked),
                        pos + fp,
                        resolver.checked,
                        fo,
                    );
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).name;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve((&self.name), pos + fp, resolver.name, fo);
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).amount;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve(
                        (&self.amount),
                        pos + fp,
                        resolver.amount,
                        fo,
                    );
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).unit;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve((&self.unit), pos + fp, resolver.unit, fo);
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).article_id;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve(
                        (&self.article_id),
                        pos + fp,
                        resolver.article_id,
                        fo,
                    );
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).alternative_article_ids;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve(
                        (&self.alternative_article_ids),
                        pos + fp,
                        resolver.alternative_article_ids,
                        fo,
                    );
                }
            }
        };
        #[automatically_derived]
        const _: () = {
            use ::rkyv::{Archive, Fallible, Serialize};
            impl<__S: Fallible + ?Sized> Serialize<__S> for Item
            where
                <Item as Identifiable>::Id: Serialize<__S>,
                bool: Serialize<__S>,
                String: Serialize<__S>,
                Option<u64>: Serialize<__S>,
                Option<Unit>: Serialize<__S>,
                Option<<Article as Identifiable>::Id>: Serialize<__S>,
                Option<Vec<<Article as Identifiable>::Id>>: Serialize<__S>,
            {
                #[inline]
                fn serialize(
                    &self,
                    serializer: &mut __S,
                ) -> ::core::result::Result<Self::Resolver, __S::Error> {
                    Ok(ItemResolver {
                        id: Serialize::<__S>::serialize(&self.id, serializer)?,
                        checked: Serialize::<__S>::serialize(&self.checked, serializer)?,
                        name: Serialize::<__S>::serialize(&self.name, serializer)?,
                        amount: Serialize::<__S>::serialize(&self.amount, serializer)?,
                        unit: Serialize::<__S>::serialize(&self.unit, serializer)?,
                        article_id: Serialize::<
                            __S,
                        >::serialize(&self.article_id, serializer)?,
                        alternative_article_ids: Serialize::<
                            __S,
                        >::serialize(&self.alternative_article_ids, serializer)?,
                    })
                }
            }
        };
        #[automatically_derived]
        const _: () = {
            use ::rkyv::{Archive, Archived, Deserialize, Fallible};
            impl<__D: Fallible + ?Sized> Deserialize<Item, __D> for Archived<Item>
            where
                <Item as Identifiable>::Id: Archive,
                Archived<
                    <Item as Identifiable>::Id,
                >: Deserialize<<Item as Identifiable>::Id, __D>,
                bool: Archive,
                Archived<bool>: Deserialize<bool, __D>,
                String: Archive,
                Archived<String>: Deserialize<String, __D>,
                Option<u64>: Archive,
                Archived<Option<u64>>: Deserialize<Option<u64>, __D>,
                Option<Unit>: Archive,
                Archived<Option<Unit>>: Deserialize<Option<Unit>, __D>,
                Option<<Article as Identifiable>::Id>: Archive,
                Archived<
                    Option<<Article as Identifiable>::Id>,
                >: Deserialize<Option<<Article as Identifiable>::Id>, __D>,
                Option<Vec<<Article as Identifiable>::Id>>: Archive,
                Archived<
                    Option<Vec<<Article as Identifiable>::Id>>,
                >: Deserialize<Option<Vec<<Article as Identifiable>::Id>>, __D>,
            {
                #[inline]
                fn deserialize(
                    &self,
                    deserializer: &mut __D,
                ) -> ::core::result::Result<Item, __D::Error> {
                    Ok(Item {
                        id: Deserialize::<
                            <Item as Identifiable>::Id,
                            __D,
                        >::deserialize(&self.id, deserializer)?,
                        checked: Deserialize::<
                            bool,
                            __D,
                        >::deserialize(&self.checked, deserializer)?,
                        name: Deserialize::<
                            String,
                            __D,
                        >::deserialize(&self.name, deserializer)?,
                        amount: Deserialize::<
                            Option<u64>,
                            __D,
                        >::deserialize(&self.amount, deserializer)?,
                        unit: Deserialize::<
                            Option<Unit>,
                            __D,
                        >::deserialize(&self.unit, deserializer)?,
                        article_id: Deserialize::<
                            Option<<Article as Identifiable>::Id>,
                            __D,
                        >::deserialize(&self.article_id, deserializer)?,
                        alternative_article_ids: Deserialize::<
                            Option<Vec<<Article as Identifiable>::Id>>,
                            __D,
                        >::deserialize(&self.alternative_article_ids, deserializer)?,
                    })
                }
            }
        };
        #[automatically_derived]
        impl ::core::clone::Clone for Item {
            #[inline]
            fn clone(&self) -> Item {
                Item {
                    id: ::core::clone::Clone::clone(&self.id),
                    checked: ::core::clone::Clone::clone(&self.checked),
                    name: ::core::clone::Clone::clone(&self.name),
                    amount: ::core::clone::Clone::clone(&self.amount),
                    unit: ::core::clone::Clone::clone(&self.unit),
                    article_id: ::core::clone::Clone::clone(&self.article_id),
                    alternative_article_ids: ::core::clone::Clone::clone(
                        &self.alternative_article_ids,
                    ),
                }
            }
        }
        #[archive_attr(derive(bytecheck::CheckBytes))]
        pub enum Unit {
            Gram,
            KiloGram,
            Pieces,
            FreeForm(String),
        }
        impl ::core::marker::StructuralPartialEq for Unit {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Unit {
            #[inline]
            fn eq(&self, other: &Unit) -> bool {
                let __self_tag = ::core::intrinsics::discriminant_value(self);
                let __arg1_tag = ::core::intrinsics::discriminant_value(other);
                __self_tag == __arg1_tag
                    && match (self, other) {
                        (Unit::FreeForm(__self_0), Unit::FreeForm(__arg1_0)) => {
                            *__self_0 == *__arg1_0
                        }
                        _ => true,
                    }
            }
            #[inline]
            fn ne(&self, other: &Unit) -> bool {
                let __self_tag = ::core::intrinsics::discriminant_value(self);
                let __arg1_tag = ::core::intrinsics::discriminant_value(other);
                __self_tag != __arg1_tag
                    || match (self, other) {
                        (Unit::FreeForm(__self_0), Unit::FreeForm(__arg1_0)) => {
                            *__self_0 != *__arg1_0
                        }
                        _ => false,
                    }
            }
        }
        #[automatically_derived]
        ///An archived [`Unit`]
        #[repr(u8)]
        pub enum ArchivedUnit
        where
            String: ::rkyv::Archive,
        {
            ///The archived counterpart of [`Unit::Gram`]
            #[allow(dead_code)]
            Gram,
            ///The archived counterpart of [`Unit::KiloGram`]
            #[allow(dead_code)]
            KiloGram,
            ///The archived counterpart of [`Unit::Pieces`]
            #[allow(dead_code)]
            Pieces,
            ///The archived counterpart of [`Unit::FreeForm`]
            #[allow(dead_code)]
            FreeForm(
                ///The archived counterpart of [`Unit::FreeForm::0`]
                ::rkyv::Archived<String>,
            ),
        }
        const _: () = {
            use ::core::{convert::Infallible, marker::PhantomData};
            use bytecheck::{
                CheckBytes, EnumCheckError, ErrorBox, StructCheckError,
                TupleStructCheckError,
            };
            #[repr(u8)]
            enum Tag {
                Gram,
                KiloGram,
                Pieces,
                FreeForm,
            }
            struct Discriminant;
            impl Discriminant {
                #[allow(non_upper_case_globals)]
                const Gram: u8 = Tag::Gram as u8;
                #[allow(non_upper_case_globals)]
                const KiloGram: u8 = Tag::KiloGram as u8;
                #[allow(non_upper_case_globals)]
                const Pieces: u8 = Tag::Pieces as u8;
                #[allow(non_upper_case_globals)]
                const FreeForm: u8 = Tag::FreeForm as u8;
            }
            #[repr(C)]
            struct VariantFreeForm(
                Tag,
                ::rkyv::Archived<String>,
                PhantomData<ArchivedUnit>,
            )
            where
                String: ::rkyv::Archive;
            impl<__C: ?Sized> CheckBytes<__C> for ArchivedUnit
            where
                String: ::rkyv::Archive,
                ::rkyv::Archived<String>: CheckBytes<__C>,
            {
                type Error = EnumCheckError<u8>;
                unsafe fn check_bytes<'__bytecheck>(
                    value: *const Self,
                    context: &mut __C,
                ) -> Result<&'__bytecheck Self, Self::Error> {
                    let tag = *value.cast::<u8>();
                    match tag {
                        Discriminant::Gram => {}
                        Discriminant::KiloGram => {}
                        Discriminant::Pieces => {}
                        Discriminant::FreeForm => {
                            let value = value.cast::<VariantFreeForm>();
                            <::rkyv::Archived<
                                String,
                            > as CheckBytes<
                                __C,
                            >>::check_bytes(&raw const (*value).1, context)
                                .map_err(|e| EnumCheckError::InvalidTuple {
                                    variant_name: "FreeForm",
                                    inner: TupleStructCheckError {
                                        field_index: 0usize,
                                        inner: ErrorBox::new(e),
                                    },
                                })?;
                        }
                        _ => return Err(EnumCheckError::InvalidTag(tag)),
                    }
                    Ok(&*value)
                }
            }
        };
        #[automatically_derived]
        ///The resolver for an archived [`Unit`]
        pub enum UnitResolver
        where
            String: ::rkyv::Archive,
        {
            ///The resolver for [`Unit::Gram`]
            #[allow(dead_code)]
            Gram,
            ///The resolver for [`Unit::KiloGram`]
            #[allow(dead_code)]
            KiloGram,
            ///The resolver for [`Unit::Pieces`]
            #[allow(dead_code)]
            Pieces,
            ///The resolver for [`Unit::FreeForm`]
            #[allow(dead_code)]
            FreeForm(
                ///The resolver for [`Unit::FreeForm::0`]
                ::rkyv::Resolver<String>,
            ),
        }
        #[automatically_derived]
        const _: () = {
            use ::core::marker::PhantomData;
            use ::rkyv::{out_field, Archive, Archived};
            #[repr(u8)]
            enum ArchivedTag {
                Gram,
                KiloGram,
                Pieces,
                FreeForm,
            }
            #[repr(C)]
            struct ArchivedVariantFreeForm(
                ArchivedTag,
                Archived<String>,
                PhantomData<Unit>,
            )
            where
                String: ::rkyv::Archive;
            impl Archive for Unit
            where
                String: ::rkyv::Archive,
            {
                type Archived = ArchivedUnit;
                type Resolver = UnitResolver;
                #[allow(clippy::unit_arg)]
                #[inline]
                unsafe fn resolve(
                    &self,
                    pos: usize,
                    resolver: Self::Resolver,
                    out: *mut Self::Archived,
                ) {
                    match resolver {
                        UnitResolver::Gram => {
                            out.cast::<ArchivedTag>().write(ArchivedTag::Gram);
                        }
                        UnitResolver::KiloGram => {
                            out.cast::<ArchivedTag>().write(ArchivedTag::KiloGram);
                        }
                        UnitResolver::Pieces => {
                            out.cast::<ArchivedTag>().write(ArchivedTag::Pieces);
                        }
                        UnitResolver::FreeForm(resolver_0) => {
                            match self {
                                Unit::FreeForm(self_0) => {
                                    let out = out.cast::<ArchivedVariantFreeForm>();
                                    (&raw mut (*out).0).write(ArchivedTag::FreeForm);
                                    let (fp, fo) = {
                                        #[allow(unused_unsafe)]
                                        unsafe {
                                            let fo = &raw mut (*out).1;
                                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                                        }
                                    };
                                    ::rkyv::Archive::resolve(self_0, pos + fp, resolver_0, fo);
                                }
                                #[allow(unreachable_patterns)]
                                _ => ::core::hint::unreachable_unchecked(),
                            }
                        }
                    }
                }
            }
        };
        #[automatically_derived]
        const _: () = {
            use ::rkyv::{Archive, Fallible, Serialize};
            impl<__S: Fallible + ?Sized> Serialize<__S> for Unit
            where
                String: Serialize<__S>,
            {
                #[inline]
                fn serialize(
                    &self,
                    serializer: &mut __S,
                ) -> ::core::result::Result<Self::Resolver, __S::Error> {
                    Ok(
                        match self {
                            Self::Gram => UnitResolver::Gram,
                            Self::KiloGram => UnitResolver::KiloGram,
                            Self::Pieces => UnitResolver::Pieces,
                            Self::FreeForm(_0) => {
                                UnitResolver::FreeForm(
                                    Serialize::<__S>::serialize(_0, serializer)?,
                                )
                            }
                        },
                    )
                }
            }
        };
        #[automatically_derived]
        const _: () = {
            use ::rkyv::{Archive, Archived, Deserialize, Fallible};
            impl<__D: Fallible + ?Sized> Deserialize<Unit, __D> for Archived<Unit>
            where
                String: Archive,
                Archived<String>: Deserialize<String, __D>,
            {
                #[inline]
                fn deserialize(
                    &self,
                    deserializer: &mut __D,
                ) -> ::core::result::Result<Unit, __D::Error> {
                    Ok(
                        match self {
                            Self::Gram => Unit::Gram,
                            Self::KiloGram => Unit::KiloGram,
                            Self::Pieces => Unit::Pieces,
                            Self::FreeForm(_0) => {
                                Unit::FreeForm(
                                    Deserialize::<String, __D>::deserialize(_0, deserializer)?,
                                )
                            }
                        },
                    )
                }
            }
        };
        #[automatically_derived]
        impl ::core::clone::Clone for Unit {
            #[inline]
            fn clone(&self) -> Unit {
                match self {
                    Unit::Gram => Unit::Gram,
                    Unit::KiloGram => Unit::KiloGram,
                    Unit::Pieces => Unit::Pieces,
                    Unit::FreeForm(__self_0) => {
                        Unit::FreeForm(::core::clone::Clone::clone(__self_0))
                    }
                }
            }
        }
        impl Identifiable for Item {
            type Id = u64;
        }
        impl PartialEq for Item {
            fn eq(&self, other: &Self) -> bool {
                self.id == other.id && self.checked == other.checked
                    && self.amount == other.amount && self.unit == other.unit
                    && self.article_id == other.article_id
                    && self.alternative_article_ids == other.alternative_article_ids
            }
        }
    }
    pub mod list {
        use rkyv::{Archive, Deserialize, Serialize};
        use super::item::Item;
        use super::shop::Shop;
        use super::Identifiable;
        #[archive_attr(derive(bytecheck::CheckBytes, Debug))]
        pub struct List {
            pub id: <List as Identifiable>::Id,
            pub name: String,
            pub shop: Option<<Shop as Identifiable>::Id>,
            pub image_id: Option<u32>,
            pub items: Vec<<Item as Identifiable>::Id>,
        }
        #[automatically_derived]
        ///An archived [`List`]
        #[repr()]
        pub struct ArchivedList
        where
            <List as Identifiable>::Id: ::rkyv::Archive,
            String: ::rkyv::Archive,
            Option<<Shop as Identifiable>::Id>: ::rkyv::Archive,
            Option<u32>: ::rkyv::Archive,
            Vec<<Item as Identifiable>::Id>: ::rkyv::Archive,
        {
            ///The archived counterpart of [`List::id`]
            pub id: ::rkyv::Archived<<List as Identifiable>::Id>,
            ///The archived counterpart of [`List::name`]
            pub name: ::rkyv::Archived<String>,
            ///The archived counterpart of [`List::shop`]
            pub shop: ::rkyv::Archived<Option<<Shop as Identifiable>::Id>>,
            ///The archived counterpart of [`List::image_id`]
            pub image_id: ::rkyv::Archived<Option<u32>>,
            ///The archived counterpart of [`List::items`]
            pub items: ::rkyv::Archived<Vec<<Item as Identifiable>::Id>>,
        }
        const _: () = {
            use ::core::{convert::Infallible, marker::PhantomData};
            use bytecheck::{
                CheckBytes, EnumCheckError, ErrorBox, StructCheckError,
                TupleStructCheckError,
            };
            impl<__C: ?Sized> CheckBytes<__C> for ArchivedList
            where
                <List as Identifiable>::Id: ::rkyv::Archive,
                String: ::rkyv::Archive,
                Option<<Shop as Identifiable>::Id>: ::rkyv::Archive,
                Option<u32>: ::rkyv::Archive,
                Vec<<Item as Identifiable>::Id>: ::rkyv::Archive,
                ::rkyv::Archived<<List as Identifiable>::Id>: CheckBytes<__C>,
                ::rkyv::Archived<String>: CheckBytes<__C>,
                ::rkyv::Archived<Option<<Shop as Identifiable>::Id>>: CheckBytes<__C>,
                ::rkyv::Archived<Option<u32>>: CheckBytes<__C>,
                ::rkyv::Archived<Vec<<Item as Identifiable>::Id>>: CheckBytes<__C>,
            {
                type Error = StructCheckError;
                unsafe fn check_bytes<'__bytecheck>(
                    value: *const Self,
                    context: &mut __C,
                ) -> Result<&'__bytecheck Self, Self::Error> {
                    let bytes = value.cast::<u8>();
                    <::rkyv::Archived<
                        <List as Identifiable>::Id,
                    > as CheckBytes<__C>>::check_bytes(&raw const (*value).id, context)
                        .map_err(|e| StructCheckError {
                            field_name: "id",
                            inner: ErrorBox::new(e),
                        })?;
                    <::rkyv::Archived<
                        String,
                    > as CheckBytes<__C>>::check_bytes(&raw const (*value).name, context)
                        .map_err(|e| StructCheckError {
                            field_name: "name",
                            inner: ErrorBox::new(e),
                        })?;
                    <::rkyv::Archived<
                        Option<<Shop as Identifiable>::Id>,
                    > as CheckBytes<__C>>::check_bytes(&raw const (*value).shop, context)
                        .map_err(|e| StructCheckError {
                            field_name: "shop",
                            inner: ErrorBox::new(e),
                        })?;
                    <::rkyv::Archived<
                        Option<u32>,
                    > as CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).image_id, context)
                        .map_err(|e| StructCheckError {
                            field_name: "image_id",
                            inner: ErrorBox::new(e),
                        })?;
                    <::rkyv::Archived<
                        Vec<<Item as Identifiable>::Id>,
                    > as CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).items, context)
                        .map_err(|e| StructCheckError {
                            field_name: "items",
                            inner: ErrorBox::new(e),
                        })?;
                    Ok(&*value)
                }
            }
        };
        #[automatically_derived]
        impl ::core::fmt::Debug for ArchivedList
        where
            <List as Identifiable>::Id: ::rkyv::Archive,
            String: ::rkyv::Archive,
            Option<<Shop as Identifiable>::Id>: ::rkyv::Archive,
            Option<u32>: ::rkyv::Archive,
            Vec<<Item as Identifiable>::Id>: ::rkyv::Archive,
        {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field5_finish(
                    f,
                    "ArchivedList",
                    "id",
                    &&self.id,
                    "name",
                    &&self.name,
                    "shop",
                    &&self.shop,
                    "image_id",
                    &&self.image_id,
                    "items",
                    &&self.items,
                )
            }
        }
        #[automatically_derived]
        ///The resolver for an archived [`List`]
        pub struct ListResolver
        where
            <List as Identifiable>::Id: ::rkyv::Archive,
            String: ::rkyv::Archive,
            Option<<Shop as Identifiable>::Id>: ::rkyv::Archive,
            Option<u32>: ::rkyv::Archive,
            Vec<<Item as Identifiable>::Id>: ::rkyv::Archive,
        {
            id: ::rkyv::Resolver<<List as Identifiable>::Id>,
            name: ::rkyv::Resolver<String>,
            shop: ::rkyv::Resolver<Option<<Shop as Identifiable>::Id>>,
            image_id: ::rkyv::Resolver<Option<u32>>,
            items: ::rkyv::Resolver<Vec<<Item as Identifiable>::Id>>,
        }
        #[automatically_derived]
        const _: () = {
            use ::core::marker::PhantomData;
            use ::rkyv::{out_field, Archive, Archived};
            impl Archive for List
            where
                <List as Identifiable>::Id: ::rkyv::Archive,
                String: ::rkyv::Archive,
                Option<<Shop as Identifiable>::Id>: ::rkyv::Archive,
                Option<u32>: ::rkyv::Archive,
                Vec<<Item as Identifiable>::Id>: ::rkyv::Archive,
            {
                type Archived = ArchivedList;
                type Resolver = ListResolver;
                #[allow(clippy::unit_arg)]
                #[inline]
                unsafe fn resolve(
                    &self,
                    pos: usize,
                    resolver: Self::Resolver,
                    out: *mut Self::Archived,
                ) {
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).id;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve((&self.id), pos + fp, resolver.id, fo);
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).name;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve((&self.name), pos + fp, resolver.name, fo);
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).shop;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve((&self.shop), pos + fp, resolver.shop, fo);
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).image_id;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve(
                        (&self.image_id),
                        pos + fp,
                        resolver.image_id,
                        fo,
                    );
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).items;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve(
                        (&self.items),
                        pos + fp,
                        resolver.items,
                        fo,
                    );
                }
            }
        };
        #[automatically_derived]
        const _: () = {
            use ::rkyv::{Archive, Fallible, Serialize};
            impl<__S: Fallible + ?Sized> Serialize<__S> for List
            where
                <List as Identifiable>::Id: Serialize<__S>,
                String: Serialize<__S>,
                Option<<Shop as Identifiable>::Id>: Serialize<__S>,
                Option<u32>: Serialize<__S>,
                Vec<<Item as Identifiable>::Id>: Serialize<__S>,
            {
                #[inline]
                fn serialize(
                    &self,
                    serializer: &mut __S,
                ) -> ::core::result::Result<Self::Resolver, __S::Error> {
                    Ok(ListResolver {
                        id: Serialize::<__S>::serialize(&self.id, serializer)?,
                        name: Serialize::<__S>::serialize(&self.name, serializer)?,
                        shop: Serialize::<__S>::serialize(&self.shop, serializer)?,
                        image_id: Serialize::<
                            __S,
                        >::serialize(&self.image_id, serializer)?,
                        items: Serialize::<__S>::serialize(&self.items, serializer)?,
                    })
                }
            }
        };
        #[automatically_derived]
        const _: () = {
            use ::rkyv::{Archive, Archived, Deserialize, Fallible};
            impl<__D: Fallible + ?Sized> Deserialize<List, __D> for Archived<List>
            where
                <List as Identifiable>::Id: Archive,
                Archived<
                    <List as Identifiable>::Id,
                >: Deserialize<<List as Identifiable>::Id, __D>,
                String: Archive,
                Archived<String>: Deserialize<String, __D>,
                Option<<Shop as Identifiable>::Id>: Archive,
                Archived<
                    Option<<Shop as Identifiable>::Id>,
                >: Deserialize<Option<<Shop as Identifiable>::Id>, __D>,
                Option<u32>: Archive,
                Archived<Option<u32>>: Deserialize<Option<u32>, __D>,
                Vec<<Item as Identifiable>::Id>: Archive,
                Archived<
                    Vec<<Item as Identifiable>::Id>,
                >: Deserialize<Vec<<Item as Identifiable>::Id>, __D>,
            {
                #[inline]
                fn deserialize(
                    &self,
                    deserializer: &mut __D,
                ) -> ::core::result::Result<List, __D::Error> {
                    Ok(List {
                        id: Deserialize::<
                            <List as Identifiable>::Id,
                            __D,
                        >::deserialize(&self.id, deserializer)?,
                        name: Deserialize::<
                            String,
                            __D,
                        >::deserialize(&self.name, deserializer)?,
                        shop: Deserialize::<
                            Option<<Shop as Identifiable>::Id>,
                            __D,
                        >::deserialize(&self.shop, deserializer)?,
                        image_id: Deserialize::<
                            Option<u32>,
                            __D,
                        >::deserialize(&self.image_id, deserializer)?,
                        items: Deserialize::<
                            Vec<<Item as Identifiable>::Id>,
                            __D,
                        >::deserialize(&self.items, deserializer)?,
                    })
                }
            }
        };
        impl ::core::marker::StructuralPartialEq for List {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for List {
            #[inline]
            fn eq(&self, other: &List) -> bool {
                self.id == other.id && self.name == other.name && self.shop == other.shop
                    && self.image_id == other.image_id && self.items == other.items
            }
            #[inline]
            fn ne(&self, other: &List) -> bool {
                self.id != other.id || self.name != other.name || self.shop != other.shop
                    || self.image_id != other.image_id || self.items != other.items
            }
        }
        #[archive_attr(derive(bytecheck::CheckBytes))]
        pub struct FlatItemsList {
            pub id: <List as Identifiable>::Id,
            pub name: String,
            pub shop: Option<<Shop as Identifiable>::Id>,
            pub image_id: Option<u32>,
            pub items: Vec<Item>,
        }
        #[automatically_derived]
        ///An archived [`FlatItemsList`]
        #[repr()]
        pub struct ArchivedFlatItemsList
        where
            <List as Identifiable>::Id: ::rkyv::Archive,
            String: ::rkyv::Archive,
            Option<<Shop as Identifiable>::Id>: ::rkyv::Archive,
            Option<u32>: ::rkyv::Archive,
            Vec<Item>: ::rkyv::Archive,
        {
            ///The archived counterpart of [`FlatItemsList::id`]
            pub id: ::rkyv::Archived<<List as Identifiable>::Id>,
            ///The archived counterpart of [`FlatItemsList::name`]
            pub name: ::rkyv::Archived<String>,
            ///The archived counterpart of [`FlatItemsList::shop`]
            pub shop: ::rkyv::Archived<Option<<Shop as Identifiable>::Id>>,
            ///The archived counterpart of [`FlatItemsList::image_id`]
            pub image_id: ::rkyv::Archived<Option<u32>>,
            ///The archived counterpart of [`FlatItemsList::items`]
            pub items: ::rkyv::Archived<Vec<Item>>,
        }
        const _: () = {
            use ::core::{convert::Infallible, marker::PhantomData};
            use bytecheck::{
                CheckBytes, EnumCheckError, ErrorBox, StructCheckError,
                TupleStructCheckError,
            };
            impl<__C: ?Sized> CheckBytes<__C> for ArchivedFlatItemsList
            where
                <List as Identifiable>::Id: ::rkyv::Archive,
                String: ::rkyv::Archive,
                Option<<Shop as Identifiable>::Id>: ::rkyv::Archive,
                Option<u32>: ::rkyv::Archive,
                Vec<Item>: ::rkyv::Archive,
                ::rkyv::Archived<<List as Identifiable>::Id>: CheckBytes<__C>,
                ::rkyv::Archived<String>: CheckBytes<__C>,
                ::rkyv::Archived<Option<<Shop as Identifiable>::Id>>: CheckBytes<__C>,
                ::rkyv::Archived<Option<u32>>: CheckBytes<__C>,
                ::rkyv::Archived<Vec<Item>>: CheckBytes<__C>,
            {
                type Error = StructCheckError;
                unsafe fn check_bytes<'__bytecheck>(
                    value: *const Self,
                    context: &mut __C,
                ) -> Result<&'__bytecheck Self, Self::Error> {
                    let bytes = value.cast::<u8>();
                    <::rkyv::Archived<
                        <List as Identifiable>::Id,
                    > as CheckBytes<__C>>::check_bytes(&raw const (*value).id, context)
                        .map_err(|e| StructCheckError {
                            field_name: "id",
                            inner: ErrorBox::new(e),
                        })?;
                    <::rkyv::Archived<
                        String,
                    > as CheckBytes<__C>>::check_bytes(&raw const (*value).name, context)
                        .map_err(|e| StructCheckError {
                            field_name: "name",
                            inner: ErrorBox::new(e),
                        })?;
                    <::rkyv::Archived<
                        Option<<Shop as Identifiable>::Id>,
                    > as CheckBytes<__C>>::check_bytes(&raw const (*value).shop, context)
                        .map_err(|e| StructCheckError {
                            field_name: "shop",
                            inner: ErrorBox::new(e),
                        })?;
                    <::rkyv::Archived<
                        Option<u32>,
                    > as CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).image_id, context)
                        .map_err(|e| StructCheckError {
                            field_name: "image_id",
                            inner: ErrorBox::new(e),
                        })?;
                    <::rkyv::Archived<
                        Vec<Item>,
                    > as CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).items, context)
                        .map_err(|e| StructCheckError {
                            field_name: "items",
                            inner: ErrorBox::new(e),
                        })?;
                    Ok(&*value)
                }
            }
        };
        #[automatically_derived]
        ///The resolver for an archived [`FlatItemsList`]
        pub struct FlatItemsListResolver
        where
            <List as Identifiable>::Id: ::rkyv::Archive,
            String: ::rkyv::Archive,
            Option<<Shop as Identifiable>::Id>: ::rkyv::Archive,
            Option<u32>: ::rkyv::Archive,
            Vec<Item>: ::rkyv::Archive,
        {
            id: ::rkyv::Resolver<<List as Identifiable>::Id>,
            name: ::rkyv::Resolver<String>,
            shop: ::rkyv::Resolver<Option<<Shop as Identifiable>::Id>>,
            image_id: ::rkyv::Resolver<Option<u32>>,
            items: ::rkyv::Resolver<Vec<Item>>,
        }
        #[automatically_derived]
        const _: () = {
            use ::core::marker::PhantomData;
            use ::rkyv::{out_field, Archive, Archived};
            impl Archive for FlatItemsList
            where
                <List as Identifiable>::Id: ::rkyv::Archive,
                String: ::rkyv::Archive,
                Option<<Shop as Identifiable>::Id>: ::rkyv::Archive,
                Option<u32>: ::rkyv::Archive,
                Vec<Item>: ::rkyv::Archive,
            {
                type Archived = ArchivedFlatItemsList;
                type Resolver = FlatItemsListResolver;
                #[allow(clippy::unit_arg)]
                #[inline]
                unsafe fn resolve(
                    &self,
                    pos: usize,
                    resolver: Self::Resolver,
                    out: *mut Self::Archived,
                ) {
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).id;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve((&self.id), pos + fp, resolver.id, fo);
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).name;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve((&self.name), pos + fp, resolver.name, fo);
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).shop;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve((&self.shop), pos + fp, resolver.shop, fo);
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).image_id;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve(
                        (&self.image_id),
                        pos + fp,
                        resolver.image_id,
                        fo,
                    );
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).items;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve(
                        (&self.items),
                        pos + fp,
                        resolver.items,
                        fo,
                    );
                }
            }
        };
        #[automatically_derived]
        const _: () = {
            use ::rkyv::{Archive, Fallible, Serialize};
            impl<__S: Fallible + ?Sized> Serialize<__S> for FlatItemsList
            where
                <List as Identifiable>::Id: Serialize<__S>,
                String: Serialize<__S>,
                Option<<Shop as Identifiable>::Id>: Serialize<__S>,
                Option<u32>: Serialize<__S>,
                Vec<Item>: Serialize<__S>,
            {
                #[inline]
                fn serialize(
                    &self,
                    serializer: &mut __S,
                ) -> ::core::result::Result<Self::Resolver, __S::Error> {
                    Ok(FlatItemsListResolver {
                        id: Serialize::<__S>::serialize(&self.id, serializer)?,
                        name: Serialize::<__S>::serialize(&self.name, serializer)?,
                        shop: Serialize::<__S>::serialize(&self.shop, serializer)?,
                        image_id: Serialize::<
                            __S,
                        >::serialize(&self.image_id, serializer)?,
                        items: Serialize::<__S>::serialize(&self.items, serializer)?,
                    })
                }
            }
        };
        #[automatically_derived]
        const _: () = {
            use ::rkyv::{Archive, Archived, Deserialize, Fallible};
            impl<__D: Fallible + ?Sized> Deserialize<FlatItemsList, __D>
            for Archived<FlatItemsList>
            where
                <List as Identifiable>::Id: Archive,
                Archived<
                    <List as Identifiable>::Id,
                >: Deserialize<<List as Identifiable>::Id, __D>,
                String: Archive,
                Archived<String>: Deserialize<String, __D>,
                Option<<Shop as Identifiable>::Id>: Archive,
                Archived<
                    Option<<Shop as Identifiable>::Id>,
                >: Deserialize<Option<<Shop as Identifiable>::Id>, __D>,
                Option<u32>: Archive,
                Archived<Option<u32>>: Deserialize<Option<u32>, __D>,
                Vec<Item>: Archive,
                Archived<Vec<Item>>: Deserialize<Vec<Item>, __D>,
            {
                #[inline]
                fn deserialize(
                    &self,
                    deserializer: &mut __D,
                ) -> ::core::result::Result<FlatItemsList, __D::Error> {
                    Ok(FlatItemsList {
                        id: Deserialize::<
                            <List as Identifiable>::Id,
                            __D,
                        >::deserialize(&self.id, deserializer)?,
                        name: Deserialize::<
                            String,
                            __D,
                        >::deserialize(&self.name, deserializer)?,
                        shop: Deserialize::<
                            Option<<Shop as Identifiable>::Id>,
                            __D,
                        >::deserialize(&self.shop, deserializer)?,
                        image_id: Deserialize::<
                            Option<u32>,
                            __D,
                        >::deserialize(&self.image_id, deserializer)?,
                        items: Deserialize::<
                            Vec<Item>,
                            __D,
                        >::deserialize(&self.items, deserializer)?,
                    })
                }
            }
        };
        impl ::core::marker::StructuralPartialEq for FlatItemsList {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for FlatItemsList {
            #[inline]
            fn eq(&self, other: &FlatItemsList) -> bool {
                self.id == other.id && self.name == other.name && self.shop == other.shop
                    && self.image_id == other.image_id && self.items == other.items
            }
            #[inline]
            fn ne(&self, other: &FlatItemsList) -> bool {
                self.id != other.id || self.name != other.name || self.shop != other.shop
                    || self.image_id != other.image_id || self.items != other.items
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for FlatItemsList {
            #[inline]
            fn clone(&self) -> FlatItemsList {
                FlatItemsList {
                    id: ::core::clone::Clone::clone(&self.id),
                    name: ::core::clone::Clone::clone(&self.name),
                    shop: ::core::clone::Clone::clone(&self.shop),
                    image_id: ::core::clone::Clone::clone(&self.image_id),
                    items: ::core::clone::Clone::clone(&self.items),
                }
            }
        }
        impl FlatItemsList {
            pub fn from_list_and_items(list: List, vec: Vec<Item>) -> Self {
                FlatItemsList {
                    id: list.id,
                    name: list.name,
                    shop: list.shop,
                    image_id: list.image_id,
                    items: vec,
                }
            }
        }
        impl Identifiable for List {
            type Id = u64;
        }
    }
    pub mod requests {
        use rkyv::{Archive, Deserialize, Serialize};
        use super::item::Item;
        /// Command-pattern based structs to be used as request parameters
        #[archive_attr(derive(bytecheck::CheckBytes))]
        pub struct StoreItemAttached {
            pub item: Item,
            pub list_id: u64,
        }
        #[automatically_derived]
        ///An archived [`StoreItemAttached`]
        #[repr()]
        pub struct ArchivedStoreItemAttached
        where
            Item: ::rkyv::Archive,
            u64: ::rkyv::Archive,
        {
            ///The archived counterpart of [`StoreItemAttached::item`]
            pub item: ::rkyv::Archived<Item>,
            ///The archived counterpart of [`StoreItemAttached::list_id`]
            pub list_id: ::rkyv::Archived<u64>,
        }
        const _: () = {
            use ::core::{convert::Infallible, marker::PhantomData};
            use bytecheck::{
                CheckBytes, EnumCheckError, ErrorBox, StructCheckError,
                TupleStructCheckError,
            };
            impl<__C: ?Sized> CheckBytes<__C> for ArchivedStoreItemAttached
            where
                Item: ::rkyv::Archive,
                u64: ::rkyv::Archive,
                ::rkyv::Archived<Item>: CheckBytes<__C>,
                ::rkyv::Archived<u64>: CheckBytes<__C>,
            {
                type Error = StructCheckError;
                unsafe fn check_bytes<'__bytecheck>(
                    value: *const Self,
                    context: &mut __C,
                ) -> Result<&'__bytecheck Self, Self::Error> {
                    let bytes = value.cast::<u8>();
                    <::rkyv::Archived<
                        Item,
                    > as CheckBytes<__C>>::check_bytes(&raw const (*value).item, context)
                        .map_err(|e| StructCheckError {
                            field_name: "item",
                            inner: ErrorBox::new(e),
                        })?;
                    <::rkyv::Archived<
                        u64,
                    > as CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).list_id, context)
                        .map_err(|e| StructCheckError {
                            field_name: "list_id",
                            inner: ErrorBox::new(e),
                        })?;
                    Ok(&*value)
                }
            }
        };
        #[automatically_derived]
        ///The resolver for an archived [`StoreItemAttached`]
        pub struct StoreItemAttachedResolver
        where
            Item: ::rkyv::Archive,
            u64: ::rkyv::Archive,
        {
            item: ::rkyv::Resolver<Item>,
            list_id: ::rkyv::Resolver<u64>,
        }
        #[automatically_derived]
        const _: () = {
            use ::core::marker::PhantomData;
            use ::rkyv::{out_field, Archive, Archived};
            impl Archive for StoreItemAttached
            where
                Item: ::rkyv::Archive,
                u64: ::rkyv::Archive,
            {
                type Archived = ArchivedStoreItemAttached;
                type Resolver = StoreItemAttachedResolver;
                #[allow(clippy::unit_arg)]
                #[inline]
                unsafe fn resolve(
                    &self,
                    pos: usize,
                    resolver: Self::Resolver,
                    out: *mut Self::Archived,
                ) {
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).item;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve((&self.item), pos + fp, resolver.item, fo);
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).list_id;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve(
                        (&self.list_id),
                        pos + fp,
                        resolver.list_id,
                        fo,
                    );
                }
            }
        };
        #[automatically_derived]
        const _: () = {
            use ::rkyv::{Archive, Fallible, Serialize};
            impl<__S: Fallible + ?Sized> Serialize<__S> for StoreItemAttached
            where
                Item: Serialize<__S>,
                u64: Serialize<__S>,
            {
                #[inline]
                fn serialize(
                    &self,
                    serializer: &mut __S,
                ) -> ::core::result::Result<Self::Resolver, __S::Error> {
                    Ok(StoreItemAttachedResolver {
                        item: Serialize::<__S>::serialize(&self.item, serializer)?,
                        list_id: Serialize::<__S>::serialize(&self.list_id, serializer)?,
                    })
                }
            }
        };
        #[automatically_derived]
        const _: () = {
            use ::rkyv::{Archive, Archived, Deserialize, Fallible};
            impl<__D: Fallible + ?Sized> Deserialize<StoreItemAttached, __D>
            for Archived<StoreItemAttached>
            where
                Item: Archive,
                Archived<Item>: Deserialize<Item, __D>,
                u64: Archive,
                Archived<u64>: Deserialize<u64, __D>,
            {
                #[inline]
                fn deserialize(
                    &self,
                    deserializer: &mut __D,
                ) -> ::core::result::Result<StoreItemAttached, __D::Error> {
                    Ok(StoreItemAttached {
                        item: Deserialize::<
                            Item,
                            __D,
                        >::deserialize(&self.item, deserializer)?,
                        list_id: Deserialize::<
                            u64,
                            __D,
                        >::deserialize(&self.list_id, deserializer)?,
                    })
                }
            }
        };
        #[archive_attr(derive(bytecheck::CheckBytes))]
        pub struct RegisterUserV1 {
            pub name: String,
            pub password: String,
        }
        #[automatically_derived]
        ///An archived [`RegisterUserV1`]
        #[repr()]
        pub struct ArchivedRegisterUserV1
        where
            String: ::rkyv::Archive,
            String: ::rkyv::Archive,
        {
            ///The archived counterpart of [`RegisterUserV1::name`]
            pub name: ::rkyv::Archived<String>,
            ///The archived counterpart of [`RegisterUserV1::password`]
            pub password: ::rkyv::Archived<String>,
        }
        const _: () = {
            use ::core::{convert::Infallible, marker::PhantomData};
            use bytecheck::{
                CheckBytes, EnumCheckError, ErrorBox, StructCheckError,
                TupleStructCheckError,
            };
            impl<__C: ?Sized> CheckBytes<__C> for ArchivedRegisterUserV1
            where
                String: ::rkyv::Archive,
                String: ::rkyv::Archive,
                ::rkyv::Archived<String>: CheckBytes<__C>,
                ::rkyv::Archived<String>: CheckBytes<__C>,
            {
                type Error = StructCheckError;
                unsafe fn check_bytes<'__bytecheck>(
                    value: *const Self,
                    context: &mut __C,
                ) -> Result<&'__bytecheck Self, Self::Error> {
                    let bytes = value.cast::<u8>();
                    <::rkyv::Archived<
                        String,
                    > as CheckBytes<__C>>::check_bytes(&raw const (*value).name, context)
                        .map_err(|e| StructCheckError {
                            field_name: "name",
                            inner: ErrorBox::new(e),
                        })?;
                    <::rkyv::Archived<
                        String,
                    > as CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).password, context)
                        .map_err(|e| StructCheckError {
                            field_name: "password",
                            inner: ErrorBox::new(e),
                        })?;
                    Ok(&*value)
                }
            }
        };
        #[automatically_derived]
        ///The resolver for an archived [`RegisterUserV1`]
        pub struct RegisterUserV1Resolver
        where
            String: ::rkyv::Archive,
            String: ::rkyv::Archive,
        {
            name: ::rkyv::Resolver<String>,
            password: ::rkyv::Resolver<String>,
        }
        #[automatically_derived]
        const _: () = {
            use ::core::marker::PhantomData;
            use ::rkyv::{out_field, Archive, Archived};
            impl Archive for RegisterUserV1
            where
                String: ::rkyv::Archive,
                String: ::rkyv::Archive,
            {
                type Archived = ArchivedRegisterUserV1;
                type Resolver = RegisterUserV1Resolver;
                #[allow(clippy::unit_arg)]
                #[inline]
                unsafe fn resolve(
                    &self,
                    pos: usize,
                    resolver: Self::Resolver,
                    out: *mut Self::Archived,
                ) {
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).name;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve((&self.name), pos + fp, resolver.name, fo);
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).password;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve(
                        (&self.password),
                        pos + fp,
                        resolver.password,
                        fo,
                    );
                }
            }
        };
        #[automatically_derived]
        const _: () = {
            use ::rkyv::{Archive, Fallible, Serialize};
            impl<__S: Fallible + ?Sized> Serialize<__S> for RegisterUserV1
            where
                String: Serialize<__S>,
                String: Serialize<__S>,
            {
                #[inline]
                fn serialize(
                    &self,
                    serializer: &mut __S,
                ) -> ::core::result::Result<Self::Resolver, __S::Error> {
                    Ok(RegisterUserV1Resolver {
                        name: Serialize::<__S>::serialize(&self.name, serializer)?,
                        password: Serialize::<
                            __S,
                        >::serialize(&self.password, serializer)?,
                    })
                }
            }
        };
        #[automatically_derived]
        const _: () = {
            use ::rkyv::{Archive, Archived, Deserialize, Fallible};
            impl<__D: Fallible + ?Sized> Deserialize<RegisterUserV1, __D>
            for Archived<RegisterUserV1>
            where
                String: Archive,
                Archived<String>: Deserialize<String, __D>,
                String: Archive,
                Archived<String>: Deserialize<String, __D>,
            {
                #[inline]
                fn deserialize(
                    &self,
                    deserializer: &mut __D,
                ) -> ::core::result::Result<RegisterUserV1, __D::Error> {
                    Ok(RegisterUserV1 {
                        name: Deserialize::<
                            String,
                            __D,
                        >::deserialize(&self.name, deserializer)?,
                        password: Deserialize::<
                            String,
                            __D,
                        >::deserialize(&self.password, deserializer)?,
                    })
                }
            }
        };
        #[archive_attr(derive(bytecheck::CheckBytes))]
        pub struct LoginUserV1 {
            pub name: String,
            pub password: String,
        }
        #[automatically_derived]
        ///An archived [`LoginUserV1`]
        #[repr()]
        pub struct ArchivedLoginUserV1
        where
            String: ::rkyv::Archive,
            String: ::rkyv::Archive,
        {
            ///The archived counterpart of [`LoginUserV1::name`]
            pub name: ::rkyv::Archived<String>,
            ///The archived counterpart of [`LoginUserV1::password`]
            pub password: ::rkyv::Archived<String>,
        }
        const _: () = {
            use ::core::{convert::Infallible, marker::PhantomData};
            use bytecheck::{
                CheckBytes, EnumCheckError, ErrorBox, StructCheckError,
                TupleStructCheckError,
            };
            impl<__C: ?Sized> CheckBytes<__C> for ArchivedLoginUserV1
            where
                String: ::rkyv::Archive,
                String: ::rkyv::Archive,
                ::rkyv::Archived<String>: CheckBytes<__C>,
                ::rkyv::Archived<String>: CheckBytes<__C>,
            {
                type Error = StructCheckError;
                unsafe fn check_bytes<'__bytecheck>(
                    value: *const Self,
                    context: &mut __C,
                ) -> Result<&'__bytecheck Self, Self::Error> {
                    let bytes = value.cast::<u8>();
                    <::rkyv::Archived<
                        String,
                    > as CheckBytes<__C>>::check_bytes(&raw const (*value).name, context)
                        .map_err(|e| StructCheckError {
                            field_name: "name",
                            inner: ErrorBox::new(e),
                        })?;
                    <::rkyv::Archived<
                        String,
                    > as CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).password, context)
                        .map_err(|e| StructCheckError {
                            field_name: "password",
                            inner: ErrorBox::new(e),
                        })?;
                    Ok(&*value)
                }
            }
        };
        #[automatically_derived]
        ///The resolver for an archived [`LoginUserV1`]
        pub struct LoginUserV1Resolver
        where
            String: ::rkyv::Archive,
            String: ::rkyv::Archive,
        {
            name: ::rkyv::Resolver<String>,
            password: ::rkyv::Resolver<String>,
        }
        #[automatically_derived]
        const _: () = {
            use ::core::marker::PhantomData;
            use ::rkyv::{out_field, Archive, Archived};
            impl Archive for LoginUserV1
            where
                String: ::rkyv::Archive,
                String: ::rkyv::Archive,
            {
                type Archived = ArchivedLoginUserV1;
                type Resolver = LoginUserV1Resolver;
                #[allow(clippy::unit_arg)]
                #[inline]
                unsafe fn resolve(
                    &self,
                    pos: usize,
                    resolver: Self::Resolver,
                    out: *mut Self::Archived,
                ) {
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).name;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve((&self.name), pos + fp, resolver.name, fo);
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).password;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve(
                        (&self.password),
                        pos + fp,
                        resolver.password,
                        fo,
                    );
                }
            }
        };
        #[automatically_derived]
        const _: () = {
            use ::rkyv::{Archive, Fallible, Serialize};
            impl<__S: Fallible + ?Sized> Serialize<__S> for LoginUserV1
            where
                String: Serialize<__S>,
                String: Serialize<__S>,
            {
                #[inline]
                fn serialize(
                    &self,
                    serializer: &mut __S,
                ) -> ::core::result::Result<Self::Resolver, __S::Error> {
                    Ok(LoginUserV1Resolver {
                        name: Serialize::<__S>::serialize(&self.name, serializer)?,
                        password: Serialize::<
                            __S,
                        >::serialize(&self.password, serializer)?,
                    })
                }
            }
        };
        #[automatically_derived]
        const _: () = {
            use ::rkyv::{Archive, Archived, Deserialize, Fallible};
            impl<__D: Fallible + ?Sized> Deserialize<LoginUserV1, __D>
            for Archived<LoginUserV1>
            where
                String: Archive,
                Archived<String>: Deserialize<String, __D>,
                String: Archive,
                Archived<String>: Deserialize<String, __D>,
            {
                #[inline]
                fn deserialize(
                    &self,
                    deserializer: &mut __D,
                ) -> ::core::result::Result<LoginUserV1, __D::Error> {
                    Ok(LoginUserV1 {
                        name: Deserialize::<
                            String,
                            __D,
                        >::deserialize(&self.name, deserializer)?,
                        password: Deserialize::<
                            String,
                            __D,
                        >::deserialize(&self.password, deserializer)?,
                    })
                }
            }
        };
    }
    pub mod shop {
        use rkyv::{Archive, Deserialize, Serialize};
        use super::Identifiable;
        #[archive_attr(derive(bytecheck::CheckBytes, Debug))]
        pub struct Shop {
            pub id: <Shop as Identifiable>::Id,
            pub name: String,
            pub image_id: Option<u32>,
        }
        #[automatically_derived]
        ///An archived [`Shop`]
        #[repr()]
        pub struct ArchivedShop
        where
            <Shop as Identifiable>::Id: ::rkyv::Archive,
            String: ::rkyv::Archive,
            Option<u32>: ::rkyv::Archive,
        {
            ///The archived counterpart of [`Shop::id`]
            pub id: ::rkyv::Archived<<Shop as Identifiable>::Id>,
            ///The archived counterpart of [`Shop::name`]
            pub name: ::rkyv::Archived<String>,
            ///The archived counterpart of [`Shop::image_id`]
            pub image_id: ::rkyv::Archived<Option<u32>>,
        }
        const _: () = {
            use ::core::{convert::Infallible, marker::PhantomData};
            use bytecheck::{
                CheckBytes, EnumCheckError, ErrorBox, StructCheckError,
                TupleStructCheckError,
            };
            impl<__C: ?Sized> CheckBytes<__C> for ArchivedShop
            where
                <Shop as Identifiable>::Id: ::rkyv::Archive,
                String: ::rkyv::Archive,
                Option<u32>: ::rkyv::Archive,
                ::rkyv::Archived<<Shop as Identifiable>::Id>: CheckBytes<__C>,
                ::rkyv::Archived<String>: CheckBytes<__C>,
                ::rkyv::Archived<Option<u32>>: CheckBytes<__C>,
            {
                type Error = StructCheckError;
                unsafe fn check_bytes<'__bytecheck>(
                    value: *const Self,
                    context: &mut __C,
                ) -> Result<&'__bytecheck Self, Self::Error> {
                    let bytes = value.cast::<u8>();
                    <::rkyv::Archived<
                        <Shop as Identifiable>::Id,
                    > as CheckBytes<__C>>::check_bytes(&raw const (*value).id, context)
                        .map_err(|e| StructCheckError {
                            field_name: "id",
                            inner: ErrorBox::new(e),
                        })?;
                    <::rkyv::Archived<
                        String,
                    > as CheckBytes<__C>>::check_bytes(&raw const (*value).name, context)
                        .map_err(|e| StructCheckError {
                            field_name: "name",
                            inner: ErrorBox::new(e),
                        })?;
                    <::rkyv::Archived<
                        Option<u32>,
                    > as CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).image_id, context)
                        .map_err(|e| StructCheckError {
                            field_name: "image_id",
                            inner: ErrorBox::new(e),
                        })?;
                    Ok(&*value)
                }
            }
        };
        #[automatically_derived]
        impl ::core::fmt::Debug for ArchivedShop
        where
            <Shop as Identifiable>::Id: ::rkyv::Archive,
            String: ::rkyv::Archive,
            Option<u32>: ::rkyv::Archive,
        {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field3_finish(
                    f,
                    "ArchivedShop",
                    "id",
                    &&self.id,
                    "name",
                    &&self.name,
                    "image_id",
                    &&self.image_id,
                )
            }
        }
        #[automatically_derived]
        ///The resolver for an archived [`Shop`]
        pub struct ShopResolver
        where
            <Shop as Identifiable>::Id: ::rkyv::Archive,
            String: ::rkyv::Archive,
            Option<u32>: ::rkyv::Archive,
        {
            id: ::rkyv::Resolver<<Shop as Identifiable>::Id>,
            name: ::rkyv::Resolver<String>,
            image_id: ::rkyv::Resolver<Option<u32>>,
        }
        #[automatically_derived]
        const _: () = {
            use ::core::marker::PhantomData;
            use ::rkyv::{out_field, Archive, Archived};
            impl Archive for Shop
            where
                <Shop as Identifiable>::Id: ::rkyv::Archive,
                String: ::rkyv::Archive,
                Option<u32>: ::rkyv::Archive,
            {
                type Archived = ArchivedShop;
                type Resolver = ShopResolver;
                #[allow(clippy::unit_arg)]
                #[inline]
                unsafe fn resolve(
                    &self,
                    pos: usize,
                    resolver: Self::Resolver,
                    out: *mut Self::Archived,
                ) {
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).id;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve((&self.id), pos + fp, resolver.id, fo);
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).name;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve((&self.name), pos + fp, resolver.name, fo);
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).image_id;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve(
                        (&self.image_id),
                        pos + fp,
                        resolver.image_id,
                        fo,
                    );
                }
            }
        };
        #[automatically_derived]
        const _: () = {
            use ::rkyv::{Archive, Fallible, Serialize};
            impl<__S: Fallible + ?Sized> Serialize<__S> for Shop
            where
                <Shop as Identifiable>::Id: Serialize<__S>,
                String: Serialize<__S>,
                Option<u32>: Serialize<__S>,
            {
                #[inline]
                fn serialize(
                    &self,
                    serializer: &mut __S,
                ) -> ::core::result::Result<Self::Resolver, __S::Error> {
                    Ok(ShopResolver {
                        id: Serialize::<__S>::serialize(&self.id, serializer)?,
                        name: Serialize::<__S>::serialize(&self.name, serializer)?,
                        image_id: Serialize::<
                            __S,
                        >::serialize(&self.image_id, serializer)?,
                    })
                }
            }
        };
        #[automatically_derived]
        const _: () = {
            use ::rkyv::{Archive, Archived, Deserialize, Fallible};
            impl<__D: Fallible + ?Sized> Deserialize<Shop, __D> for Archived<Shop>
            where
                <Shop as Identifiable>::Id: Archive,
                Archived<
                    <Shop as Identifiable>::Id,
                >: Deserialize<<Shop as Identifiable>::Id, __D>,
                String: Archive,
                Archived<String>: Deserialize<String, __D>,
                Option<u32>: Archive,
                Archived<Option<u32>>: Deserialize<Option<u32>, __D>,
            {
                #[inline]
                fn deserialize(
                    &self,
                    deserializer: &mut __D,
                ) -> ::core::result::Result<Shop, __D::Error> {
                    Ok(Shop {
                        id: Deserialize::<
                            <Shop as Identifiable>::Id,
                            __D,
                        >::deserialize(&self.id, deserializer)?,
                        name: Deserialize::<
                            String,
                            __D,
                        >::deserialize(&self.name, deserializer)?,
                        image_id: Deserialize::<
                            Option<u32>,
                            __D,
                        >::deserialize(&self.image_id, deserializer)?,
                    })
                }
            }
        };
        impl Identifiable for Shop {
            type Id = u64;
        }
    }
    pub mod user {
        use std::any::TypeId;
        use rkyv::{Archive, Deserialize, Serialize};
        use super::Identifiable;
        #[archive_attr(derive(bytecheck::CheckBytes))]
        pub struct User {
            pub id: <Self as Identifiable>::Id,
            pub name: String,
            pub profile_picture_id: Option<u64>,
        }
        #[automatically_derived]
        ///An archived [`User`]
        #[repr()]
        pub struct ArchivedUser
        where
            <User as Identifiable>::Id: ::rkyv::Archive,
            String: ::rkyv::Archive,
            Option<u64>: ::rkyv::Archive,
        {
            ///The archived counterpart of [`User::id`]
            pub id: ::rkyv::Archived<<User as Identifiable>::Id>,
            ///The archived counterpart of [`User::name`]
            pub name: ::rkyv::Archived<String>,
            ///The archived counterpart of [`User::profile_picture_id`]
            pub profile_picture_id: ::rkyv::Archived<Option<u64>>,
        }
        const _: () = {
            use ::core::{convert::Infallible, marker::PhantomData};
            use bytecheck::{
                CheckBytes, EnumCheckError, ErrorBox, StructCheckError,
                TupleStructCheckError,
            };
            impl<__C: ?Sized> CheckBytes<__C> for ArchivedUser
            where
                <User as Identifiable>::Id: ::rkyv::Archive,
                String: ::rkyv::Archive,
                Option<u64>: ::rkyv::Archive,
                ::rkyv::Archived<<User as Identifiable>::Id>: CheckBytes<__C>,
                ::rkyv::Archived<String>: CheckBytes<__C>,
                ::rkyv::Archived<Option<u64>>: CheckBytes<__C>,
            {
                type Error = StructCheckError;
                unsafe fn check_bytes<'__bytecheck>(
                    value: *const Self,
                    context: &mut __C,
                ) -> Result<&'__bytecheck Self, Self::Error> {
                    let bytes = value.cast::<u8>();
                    <::rkyv::Archived<
                        <User as Identifiable>::Id,
                    > as CheckBytes<__C>>::check_bytes(&raw const (*value).id, context)
                        .map_err(|e| StructCheckError {
                            field_name: "id",
                            inner: ErrorBox::new(e),
                        })?;
                    <::rkyv::Archived<
                        String,
                    > as CheckBytes<__C>>::check_bytes(&raw const (*value).name, context)
                        .map_err(|e| StructCheckError {
                            field_name: "name",
                            inner: ErrorBox::new(e),
                        })?;
                    <::rkyv::Archived<
                        Option<u64>,
                    > as CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).profile_picture_id, context)
                        .map_err(|e| StructCheckError {
                            field_name: "profile_picture_id",
                            inner: ErrorBox::new(e),
                        })?;
                    Ok(&*value)
                }
            }
        };
        #[automatically_derived]
        ///The resolver for an archived [`User`]
        pub struct UserResolver
        where
            <User as Identifiable>::Id: ::rkyv::Archive,
            String: ::rkyv::Archive,
            Option<u64>: ::rkyv::Archive,
        {
            id: ::rkyv::Resolver<<User as Identifiable>::Id>,
            name: ::rkyv::Resolver<String>,
            profile_picture_id: ::rkyv::Resolver<Option<u64>>,
        }
        #[automatically_derived]
        const _: () = {
            use ::core::marker::PhantomData;
            use ::rkyv::{out_field, Archive, Archived};
            impl Archive for User
            where
                <User as Identifiable>::Id: ::rkyv::Archive,
                String: ::rkyv::Archive,
                Option<u64>: ::rkyv::Archive,
            {
                type Archived = ArchivedUser;
                type Resolver = UserResolver;
                #[allow(clippy::unit_arg)]
                #[inline]
                unsafe fn resolve(
                    &self,
                    pos: usize,
                    resolver: Self::Resolver,
                    out: *mut Self::Archived,
                ) {
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).id;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve((&self.id), pos + fp, resolver.id, fo);
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).name;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve((&self.name), pos + fp, resolver.name, fo);
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).profile_picture_id;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve(
                        (&self.profile_picture_id),
                        pos + fp,
                        resolver.profile_picture_id,
                        fo,
                    );
                }
            }
        };
        #[automatically_derived]
        const _: () = {
            use ::rkyv::{Archive, Fallible, Serialize};
            impl<__S: Fallible + ?Sized> Serialize<__S> for User
            where
                <User as Identifiable>::Id: Serialize<__S>,
                String: Serialize<__S>,
                Option<u64>: Serialize<__S>,
            {
                #[inline]
                fn serialize(
                    &self,
                    serializer: &mut __S,
                ) -> ::core::result::Result<Self::Resolver, __S::Error> {
                    Ok(UserResolver {
                        id: Serialize::<__S>::serialize(&self.id, serializer)?,
                        name: Serialize::<__S>::serialize(&self.name, serializer)?,
                        profile_picture_id: Serialize::<
                            __S,
                        >::serialize(&self.profile_picture_id, serializer)?,
                    })
                }
            }
        };
        #[automatically_derived]
        const _: () = {
            use ::rkyv::{Archive, Archived, Deserialize, Fallible};
            impl<__D: Fallible + ?Sized> Deserialize<User, __D> for Archived<User>
            where
                <User as Identifiable>::Id: Archive,
                Archived<
                    <User as Identifiable>::Id,
                >: Deserialize<<User as Identifiable>::Id, __D>,
                String: Archive,
                Archived<String>: Deserialize<String, __D>,
                Option<u64>: Archive,
                Archived<Option<u64>>: Deserialize<Option<u64>, __D>,
            {
                #[inline]
                fn deserialize(
                    &self,
                    deserializer: &mut __D,
                ) -> ::core::result::Result<User, __D::Error> {
                    Ok(User {
                        id: Deserialize::<
                            <User as Identifiable>::Id,
                            __D,
                        >::deserialize(&self.id, deserializer)?,
                        name: Deserialize::<
                            String,
                            __D,
                        >::deserialize(&self.name, deserializer)?,
                        profile_picture_id: Deserialize::<
                            Option<u64>,
                            __D,
                        >::deserialize(&self.profile_picture_id, deserializer)?,
                    })
                }
            }
        };
        #[automatically_derived]
        impl ::core::fmt::Debug for User {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field3_finish(
                    f,
                    "User",
                    "id",
                    &&self.id,
                    "name",
                    &&self.name,
                    "profile_picture_id",
                    &&self.profile_picture_id,
                )
            }
        }
        #[archive_attr(derive(bytecheck::CheckBytes))]
        pub struct UserWithPassword {
            pub user: User,
            pub password: Password,
        }
        #[automatically_derived]
        ///An archived [`UserWithPassword`]
        #[repr()]
        pub struct ArchivedUserWithPassword
        where
            User: ::rkyv::Archive,
            Password: ::rkyv::Archive,
        {
            ///The archived counterpart of [`UserWithPassword::user`]
            pub user: ::rkyv::Archived<User>,
            ///The archived counterpart of [`UserWithPassword::password`]
            pub password: ::rkyv::Archived<Password>,
        }
        const _: () = {
            use ::core::{convert::Infallible, marker::PhantomData};
            use bytecheck::{
                CheckBytes, EnumCheckError, ErrorBox, StructCheckError,
                TupleStructCheckError,
            };
            impl<__C: ?Sized> CheckBytes<__C> for ArchivedUserWithPassword
            where
                User: ::rkyv::Archive,
                Password: ::rkyv::Archive,
                ::rkyv::Archived<User>: CheckBytes<__C>,
                ::rkyv::Archived<Password>: CheckBytes<__C>,
            {
                type Error = StructCheckError;
                unsafe fn check_bytes<'__bytecheck>(
                    value: *const Self,
                    context: &mut __C,
                ) -> Result<&'__bytecheck Self, Self::Error> {
                    let bytes = value.cast::<u8>();
                    <::rkyv::Archived<
                        User,
                    > as CheckBytes<__C>>::check_bytes(&raw const (*value).user, context)
                        .map_err(|e| StructCheckError {
                            field_name: "user",
                            inner: ErrorBox::new(e),
                        })?;
                    <::rkyv::Archived<
                        Password,
                    > as CheckBytes<
                        __C,
                    >>::check_bytes(&raw const (*value).password, context)
                        .map_err(|e| StructCheckError {
                            field_name: "password",
                            inner: ErrorBox::new(e),
                        })?;
                    Ok(&*value)
                }
            }
        };
        #[automatically_derived]
        ///The resolver for an archived [`UserWithPassword`]
        pub struct UserWithPasswordResolver
        where
            User: ::rkyv::Archive,
            Password: ::rkyv::Archive,
        {
            user: ::rkyv::Resolver<User>,
            password: ::rkyv::Resolver<Password>,
        }
        #[automatically_derived]
        const _: () = {
            use ::core::marker::PhantomData;
            use ::rkyv::{out_field, Archive, Archived};
            impl Archive for UserWithPassword
            where
                User: ::rkyv::Archive,
                Password: ::rkyv::Archive,
            {
                type Archived = ArchivedUserWithPassword;
                type Resolver = UserWithPasswordResolver;
                #[allow(clippy::unit_arg)]
                #[inline]
                unsafe fn resolve(
                    &self,
                    pos: usize,
                    resolver: Self::Resolver,
                    out: *mut Self::Archived,
                ) {
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).user;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve((&self.user), pos + fp, resolver.user, fo);
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).password;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve(
                        (&self.password),
                        pos + fp,
                        resolver.password,
                        fo,
                    );
                }
            }
        };
        #[automatically_derived]
        const _: () = {
            use ::rkyv::{Archive, Fallible, Serialize};
            impl<__S: Fallible + ?Sized> Serialize<__S> for UserWithPassword
            where
                User: Serialize<__S>,
                Password: Serialize<__S>,
            {
                #[inline]
                fn serialize(
                    &self,
                    serializer: &mut __S,
                ) -> ::core::result::Result<Self::Resolver, __S::Error> {
                    Ok(UserWithPasswordResolver {
                        user: Serialize::<__S>::serialize(&self.user, serializer)?,
                        password: Serialize::<
                            __S,
                        >::serialize(&self.password, serializer)?,
                    })
                }
            }
        };
        #[automatically_derived]
        const _: () = {
            use ::rkyv::{Archive, Archived, Deserialize, Fallible};
            impl<__D: Fallible + ?Sized> Deserialize<UserWithPassword, __D>
            for Archived<UserWithPassword>
            where
                User: Archive,
                Archived<User>: Deserialize<User, __D>,
                Password: Archive,
                Archived<Password>: Deserialize<Password, __D>,
            {
                #[inline]
                fn deserialize(
                    &self,
                    deserializer: &mut __D,
                ) -> ::core::result::Result<UserWithPassword, __D::Error> {
                    Ok(UserWithPassword {
                        user: Deserialize::<
                            User,
                            __D,
                        >::deserialize(&self.user, deserializer)?,
                        password: Deserialize::<
                            Password,
                            __D,
                        >::deserialize(&self.password, deserializer)?,
                    })
                }
            }
        };
        #[automatically_derived]
        impl ::core::fmt::Debug for UserWithPassword {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "UserWithPassword",
                    "user",
                    &&self.user,
                    "password",
                    &&self.password,
                )
            }
        }
        #[archive_attr(derive(bytecheck::CheckBytes))]
        pub struct Password {
            pub hash: Vec<u8>,
            pub salt: Vec<u8>,
        }
        #[automatically_derived]
        ///An archived [`Password`]
        #[repr()]
        pub struct ArchivedPassword
        where
            Vec<u8>: ::rkyv::Archive,
            Vec<u8>: ::rkyv::Archive,
        {
            ///The archived counterpart of [`Password::hash`]
            pub hash: ::rkyv::Archived<Vec<u8>>,
            ///The archived counterpart of [`Password::salt`]
            pub salt: ::rkyv::Archived<Vec<u8>>,
        }
        const _: () = {
            use ::core::{convert::Infallible, marker::PhantomData};
            use bytecheck::{
                CheckBytes, EnumCheckError, ErrorBox, StructCheckError,
                TupleStructCheckError,
            };
            impl<__C: ?Sized> CheckBytes<__C> for ArchivedPassword
            where
                Vec<u8>: ::rkyv::Archive,
                Vec<u8>: ::rkyv::Archive,
                ::rkyv::Archived<Vec<u8>>: CheckBytes<__C>,
                ::rkyv::Archived<Vec<u8>>: CheckBytes<__C>,
            {
                type Error = StructCheckError;
                unsafe fn check_bytes<'__bytecheck>(
                    value: *const Self,
                    context: &mut __C,
                ) -> Result<&'__bytecheck Self, Self::Error> {
                    let bytes = value.cast::<u8>();
                    <::rkyv::Archived<
                        Vec<u8>,
                    > as CheckBytes<__C>>::check_bytes(&raw const (*value).hash, context)
                        .map_err(|e| StructCheckError {
                            field_name: "hash",
                            inner: ErrorBox::new(e),
                        })?;
                    <::rkyv::Archived<
                        Vec<u8>,
                    > as CheckBytes<__C>>::check_bytes(&raw const (*value).salt, context)
                        .map_err(|e| StructCheckError {
                            field_name: "salt",
                            inner: ErrorBox::new(e),
                        })?;
                    Ok(&*value)
                }
            }
        };
        #[automatically_derived]
        ///The resolver for an archived [`Password`]
        pub struct PasswordResolver
        where
            Vec<u8>: ::rkyv::Archive,
            Vec<u8>: ::rkyv::Archive,
        {
            hash: ::rkyv::Resolver<Vec<u8>>,
            salt: ::rkyv::Resolver<Vec<u8>>,
        }
        #[automatically_derived]
        const _: () = {
            use ::core::marker::PhantomData;
            use ::rkyv::{out_field, Archive, Archived};
            impl Archive for Password
            where
                Vec<u8>: ::rkyv::Archive,
                Vec<u8>: ::rkyv::Archive,
            {
                type Archived = ArchivedPassword;
                type Resolver = PasswordResolver;
                #[allow(clippy::unit_arg)]
                #[inline]
                unsafe fn resolve(
                    &self,
                    pos: usize,
                    resolver: Self::Resolver,
                    out: *mut Self::Archived,
                ) {
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).hash;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve((&self.hash), pos + fp, resolver.hash, fo);
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).salt;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve((&self.salt), pos + fp, resolver.salt, fo);
                }
            }
        };
        #[automatically_derived]
        const _: () = {
            use ::rkyv::{Archive, Fallible, Serialize};
            impl<__S: Fallible + ?Sized> Serialize<__S> for Password
            where
                Vec<u8>: Serialize<__S>,
                Vec<u8>: Serialize<__S>,
            {
                #[inline]
                fn serialize(
                    &self,
                    serializer: &mut __S,
                ) -> ::core::result::Result<Self::Resolver, __S::Error> {
                    Ok(PasswordResolver {
                        hash: Serialize::<__S>::serialize(&self.hash, serializer)?,
                        salt: Serialize::<__S>::serialize(&self.salt, serializer)?,
                    })
                }
            }
        };
        #[automatically_derived]
        const _: () = {
            use ::rkyv::{Archive, Archived, Deserialize, Fallible};
            impl<__D: Fallible + ?Sized> Deserialize<Password, __D>
            for Archived<Password>
            where
                Vec<u8>: Archive,
                Archived<Vec<u8>>: Deserialize<Vec<u8>, __D>,
                Vec<u8>: Archive,
                Archived<Vec<u8>>: Deserialize<Vec<u8>, __D>,
            {
                #[inline]
                fn deserialize(
                    &self,
                    deserializer: &mut __D,
                ) -> ::core::result::Result<Password, __D::Error> {
                    Ok(Password {
                        hash: Deserialize::<
                            Vec<u8>,
                            __D,
                        >::deserialize(&self.hash, deserializer)?,
                        salt: Deserialize::<
                            Vec<u8>,
                            __D,
                        >::deserialize(&self.salt, deserializer)?,
                    })
                }
            }
        };
        #[automatically_derived]
        impl ::core::fmt::Debug for Password {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "Password",
                    "hash",
                    &&self.hash,
                    "salt",
                    &&self.salt,
                )
            }
        }
        impl Identifiable for User {
            type Id = u64;
        }
        #[archive_attr(derive(bytecheck::CheckBytes))]
        pub struct ObjectList {
            pub typ: u64,
            pub list: Vec<u64>,
        }
        #[automatically_derived]
        ///An archived [`ObjectList`]
        #[repr()]
        pub struct ArchivedObjectList
        where
            u64: ::rkyv::Archive,
            Vec<u64>: ::rkyv::Archive,
        {
            ///The archived counterpart of [`ObjectList::typ`]
            pub typ: ::rkyv::Archived<u64>,
            ///The archived counterpart of [`ObjectList::list`]
            pub list: ::rkyv::Archived<Vec<u64>>,
        }
        const _: () = {
            use ::core::{convert::Infallible, marker::PhantomData};
            use bytecheck::{
                CheckBytes, EnumCheckError, ErrorBox, StructCheckError,
                TupleStructCheckError,
            };
            impl<__C: ?Sized> CheckBytes<__C> for ArchivedObjectList
            where
                u64: ::rkyv::Archive,
                Vec<u64>: ::rkyv::Archive,
                ::rkyv::Archived<u64>: CheckBytes<__C>,
                ::rkyv::Archived<Vec<u64>>: CheckBytes<__C>,
            {
                type Error = StructCheckError;
                unsafe fn check_bytes<'__bytecheck>(
                    value: *const Self,
                    context: &mut __C,
                ) -> Result<&'__bytecheck Self, Self::Error> {
                    let bytes = value.cast::<u8>();
                    <::rkyv::Archived<
                        u64,
                    > as CheckBytes<__C>>::check_bytes(&raw const (*value).typ, context)
                        .map_err(|e| StructCheckError {
                            field_name: "typ",
                            inner: ErrorBox::new(e),
                        })?;
                    <::rkyv::Archived<
                        Vec<u64>,
                    > as CheckBytes<__C>>::check_bytes(&raw const (*value).list, context)
                        .map_err(|e| StructCheckError {
                            field_name: "list",
                            inner: ErrorBox::new(e),
                        })?;
                    Ok(&*value)
                }
            }
        };
        #[automatically_derived]
        ///The resolver for an archived [`ObjectList`]
        pub struct ObjectListResolver
        where
            u64: ::rkyv::Archive,
            Vec<u64>: ::rkyv::Archive,
        {
            typ: ::rkyv::Resolver<u64>,
            list: ::rkyv::Resolver<Vec<u64>>,
        }
        #[automatically_derived]
        const _: () = {
            use ::core::marker::PhantomData;
            use ::rkyv::{out_field, Archive, Archived};
            impl Archive for ObjectList
            where
                u64: ::rkyv::Archive,
                Vec<u64>: ::rkyv::Archive,
            {
                type Archived = ArchivedObjectList;
                type Resolver = ObjectListResolver;
                #[allow(clippy::unit_arg)]
                #[inline]
                unsafe fn resolve(
                    &self,
                    pos: usize,
                    resolver: Self::Resolver,
                    out: *mut Self::Archived,
                ) {
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).typ;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve((&self.typ), pos + fp, resolver.typ, fo);
                    let (fp, fo) = {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let fo = &raw mut (*out).list;
                            (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                        }
                    };
                    ::rkyv::Archive::resolve((&self.list), pos + fp, resolver.list, fo);
                }
            }
        };
        #[automatically_derived]
        const _: () = {
            use ::rkyv::{Archive, Fallible, Serialize};
            impl<__S: Fallible + ?Sized> Serialize<__S> for ObjectList
            where
                u64: Serialize<__S>,
                Vec<u64>: Serialize<__S>,
            {
                #[inline]
                fn serialize(
                    &self,
                    serializer: &mut __S,
                ) -> ::core::result::Result<Self::Resolver, __S::Error> {
                    Ok(ObjectListResolver {
                        typ: Serialize::<__S>::serialize(&self.typ, serializer)?,
                        list: Serialize::<__S>::serialize(&self.list, serializer)?,
                    })
                }
            }
        };
        #[automatically_derived]
        const _: () = {
            use ::rkyv::{Archive, Archived, Deserialize, Fallible};
            impl<__D: Fallible + ?Sized> Deserialize<ObjectList, __D>
            for Archived<ObjectList>
            where
                u64: Archive,
                Archived<u64>: Deserialize<u64, __D>,
                Vec<u64>: Archive,
                Archived<Vec<u64>>: Deserialize<Vec<u64>, __D>,
            {
                #[inline]
                fn deserialize(
                    &self,
                    deserializer: &mut __D,
                ) -> ::core::result::Result<ObjectList, __D::Error> {
                    Ok(ObjectList {
                        typ: Deserialize::<
                            u64,
                            __D,
                        >::deserialize(&self.typ, deserializer)?,
                        list: Deserialize::<
                            Vec<u64>,
                            __D,
                        >::deserialize(&self.list, deserializer)?,
                    })
                }
            }
        };
        #[automatically_derived]
        impl ::core::fmt::Debug for ObjectList {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "ObjectList",
                    "typ",
                    &&self.typ,
                    "list",
                    &&self.list,
                )
            }
        }
    }
    pub trait MarkerTrait<T: Identifiable> {
        fn try_from_u8_slice() -> Result<<T as Identifiable>::Id, &'static str>;
    }
    /// Declares the type of the Id of the implementing struct. Note that the Id still needs to be manually implemented.
    ///
    ///  The trait serves tight coupling between model objects to prevent divergence (for example in Database objects) when modifying id type later.
    pub trait Identifiable {
        type Id: Sized
            + PartialEq
            + Eq
            + rkyv::Serialize<SharedDeserializeMap>
            + AsBytes
            + Clone;
    }
    /// Access-control-list for all kinds of data objects. Warning: if your ids are generated in an overlapping way, you must seperate the AccessControlLists in seperate DBs/keyspaces
    #[archive_attr(derive(bytecheck::CheckBytes))]
    pub struct AccessControlList<Object: Identifiable, User: Identifiable> {
        pub object_id: Object::Id,
        pub owner: User::Id,
        pub allowed_user_ids: Vec<User::Id>,
    }
    #[automatically_derived]
    ///An archived [`AccessControlList`]
    #[repr()]
    pub struct ArchivedAccessControlList<Object: Identifiable, User: Identifiable>
    where
        Object::Id: ::rkyv::Archive,
        User::Id: ::rkyv::Archive,
        Vec<User::Id>: ::rkyv::Archive,
    {
        ///The archived counterpart of [`AccessControlList::object_id`]
        pub object_id: ::rkyv::Archived<Object::Id>,
        ///The archived counterpart of [`AccessControlList::owner`]
        pub owner: ::rkyv::Archived<User::Id>,
        ///The archived counterpart of [`AccessControlList::allowed_user_ids`]
        pub allowed_user_ids: ::rkyv::Archived<Vec<User::Id>>,
    }
    const _: () = {
        use ::core::{convert::Infallible, marker::PhantomData};
        use bytecheck::{
            CheckBytes, EnumCheckError, ErrorBox, StructCheckError, TupleStructCheckError,
        };
        impl<__C: ?Sized, Object: Identifiable, User: Identifiable> CheckBytes<__C>
        for ArchivedAccessControlList<Object, User>
        where
            Object::Id: ::rkyv::Archive,
            User::Id: ::rkyv::Archive,
            Vec<User::Id>: ::rkyv::Archive,
            ::rkyv::Archived<Object::Id>: CheckBytes<__C>,
            ::rkyv::Archived<User::Id>: CheckBytes<__C>,
            ::rkyv::Archived<Vec<User::Id>>: CheckBytes<__C>,
        {
            type Error = StructCheckError;
            unsafe fn check_bytes<'__bytecheck>(
                value: *const Self,
                context: &mut __C,
            ) -> Result<&'__bytecheck Self, Self::Error> {
                let bytes = value.cast::<u8>();
                <::rkyv::Archived<
                    Object::Id,
                > as CheckBytes<
                    __C,
                >>::check_bytes(&raw const (*value).object_id, context)
                    .map_err(|e| StructCheckError {
                        field_name: "object_id",
                        inner: ErrorBox::new(e),
                    })?;
                <::rkyv::Archived<
                    User::Id,
                > as CheckBytes<__C>>::check_bytes(&raw const (*value).owner, context)
                    .map_err(|e| StructCheckError {
                        field_name: "owner",
                        inner: ErrorBox::new(e),
                    })?;
                <::rkyv::Archived<
                    Vec<User::Id>,
                > as CheckBytes<
                    __C,
                >>::check_bytes(&raw const (*value).allowed_user_ids, context)
                    .map_err(|e| StructCheckError {
                        field_name: "allowed_user_ids",
                        inner: ErrorBox::new(e),
                    })?;
                Ok(&*value)
            }
        }
    };
    #[automatically_derived]
    ///The resolver for an archived [`AccessControlList`]
    pub struct AccessControlListResolver<Object: Identifiable, User: Identifiable>
    where
        Object::Id: ::rkyv::Archive,
        User::Id: ::rkyv::Archive,
        Vec<User::Id>: ::rkyv::Archive,
    {
        object_id: ::rkyv::Resolver<Object::Id>,
        owner: ::rkyv::Resolver<User::Id>,
        allowed_user_ids: ::rkyv::Resolver<Vec<User::Id>>,
    }
    #[automatically_derived]
    const _: () = {
        use ::core::marker::PhantomData;
        use ::rkyv::{out_field, Archive, Archived};
        impl<Object: Identifiable, User: Identifiable> Archive
        for AccessControlList<Object, User>
        where
            Object::Id: ::rkyv::Archive,
            User::Id: ::rkyv::Archive,
            Vec<User::Id>: ::rkyv::Archive,
        {
            type Archived = ArchivedAccessControlList<Object, User>;
            type Resolver = AccessControlListResolver<Object, User>;
            #[allow(clippy::unit_arg)]
            #[inline]
            unsafe fn resolve(
                &self,
                pos: usize,
                resolver: Self::Resolver,
                out: *mut Self::Archived,
            ) {
                let (fp, fo) = {
                    #[allow(unused_unsafe)]
                    unsafe {
                        let fo = &raw mut (*out).object_id;
                        (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                    }
                };
                ::rkyv::Archive::resolve(
                    (&self.object_id),
                    pos + fp,
                    resolver.object_id,
                    fo,
                );
                let (fp, fo) = {
                    #[allow(unused_unsafe)]
                    unsafe {
                        let fo = &raw mut (*out).owner;
                        (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                    }
                };
                ::rkyv::Archive::resolve((&self.owner), pos + fp, resolver.owner, fo);
                let (fp, fo) = {
                    #[allow(unused_unsafe)]
                    unsafe {
                        let fo = &raw mut (*out).allowed_user_ids;
                        (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
                    }
                };
                ::rkyv::Archive::resolve(
                    (&self.allowed_user_ids),
                    pos + fp,
                    resolver.allowed_user_ids,
                    fo,
                );
            }
        }
    };
    #[automatically_derived]
    const _: () = {
        use ::rkyv::{Archive, Fallible, Serialize};
        impl<
            __S: Fallible + ?Sized,
            Object: Identifiable,
            User: Identifiable,
        > Serialize<__S> for AccessControlList<Object, User>
        where
            Object::Id: Serialize<__S>,
            User::Id: Serialize<__S>,
            Vec<User::Id>: Serialize<__S>,
        {
            #[inline]
            fn serialize(
                &self,
                serializer: &mut __S,
            ) -> ::core::result::Result<Self::Resolver, __S::Error> {
                Ok(AccessControlListResolver {
                    object_id: Serialize::<__S>::serialize(&self.object_id, serializer)?,
                    owner: Serialize::<__S>::serialize(&self.owner, serializer)?,
                    allowed_user_ids: Serialize::<
                        __S,
                    >::serialize(&self.allowed_user_ids, serializer)?,
                })
            }
        }
    };
    #[automatically_derived]
    const _: () = {
        use ::rkyv::{Archive, Archived, Deserialize, Fallible};
        impl<
            __D: Fallible + ?Sized,
            Object: Identifiable,
            User: Identifiable,
        > Deserialize<AccessControlList<Object, User>, __D>
        for Archived<AccessControlList<Object, User>>
        where
            Object::Id: Archive,
            Archived<Object::Id>: Deserialize<Object::Id, __D>,
            User::Id: Archive,
            Archived<User::Id>: Deserialize<User::Id, __D>,
            Vec<User::Id>: Archive,
            Archived<Vec<User::Id>>: Deserialize<Vec<User::Id>, __D>,
        {
            #[inline]
            fn deserialize(
                &self,
                deserializer: &mut __D,
            ) -> ::core::result::Result<AccessControlList<Object, User>, __D::Error> {
                Ok(AccessControlList {
                    object_id: Deserialize::<
                        Object::Id,
                        __D,
                    >::deserialize(&self.object_id, deserializer)?,
                    owner: Deserialize::<
                        User::Id,
                        __D,
                    >::deserialize(&self.owner, deserializer)?,
                    allowed_user_ids: Deserialize::<
                        Vec<User::Id>,
                        __D,
                    >::deserialize(&self.allowed_user_ids, deserializer)?,
                })
            }
        }
    };
}
