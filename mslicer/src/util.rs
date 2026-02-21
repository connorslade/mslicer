#[macro_export]
macro_rules! include_asset {
    ($name:expr) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $name))
    };
}

/// Generates a 'ref' type given an app field and it's type. This allows for
/// safely calling methods requiring an App reference on values stored in App.
#[macro_export]
macro_rules! app_ref_type {
    ($type:ident, $field:ident) => {
        concat_idents::concat_idents!(type_ref = $type, Ref {
            pub struct type_ref<'a> {
                app: &'a mut App,
            }

            impl App {
                #[inline(always)]
                pub fn $field(&mut self) -> type_ref<'_> {
                    type_ref::new(self)
                }
            }

            impl<'a> type_ref<'a> {
                pub fn new(app: &'a mut App) -> Self {
                    Self { app }
                }
            }

            impl<'a> std::ops::Deref for type_ref<'a> {
                type Target = $type;

                fn deref(&self) -> &Self::Target {
                    &self.app.$field
                }
            }

            impl<'a> std::ops::DerefMut for type_ref<'a> {
                fn deref_mut(&mut self) -> &mut $type {
                    &mut self.app.$field
                }
            }
        });
    };
}
