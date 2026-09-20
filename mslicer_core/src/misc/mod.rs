pub mod management;
pub mod render;

#[macro_export]
macro_rules! core_ref_type {
    ($type:ident, $field:ident) => {
        concat_idents::concat_idents!(type_ref = $type, Ref {
            pub struct type_ref<'a> {
                core: &'a mut $crate::core::Core,
            }

            impl $crate::core::Core {
                #[inline(always)]
                pub fn $field(&mut self) -> type_ref<'_> {
                    type_ref::new(self)
                }
            }

            impl<'a> type_ref<'a> {
                pub fn new(core: &'a mut $crate::core::Core) -> Self {
                    Self { core }
                }
            }

            impl<'a> std::ops::Deref for type_ref<'a> {
                type Target = $type;

                fn deref(&self) -> &Self::Target {
                    &self.core.$field
                }
            }

            impl<'a> std::ops::DerefMut for type_ref<'a> {
                fn deref_mut(&mut self) -> &mut $type {
                    &mut self.core.$field
                }
            }
        });
    };
}
