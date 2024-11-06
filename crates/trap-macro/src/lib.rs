mod riscv;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use riscv::KERNEL_TRAP;
use syn::{parse_macro_input, ItemFn, LitStr, MetaNameValue};

#[proc_macro_attribute]
pub fn kernel_trap(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr: MetaNameValue = parse_macro_input!(attr);
    let attr_path = attr.path.get_ident().unwrap();
    if attr_path.to_string() != "context" {
        panic!("attribute set error");
    }
    let context_type = attr.value;
    let mut handler_impl = parse_macro_input!(item as ItemFn);
    let handler_name = handler_impl.sig.ident.clone();
    let handler_impl_name = format_ident!("{}_impl", handler_impl.sig.ident);
    let handler_vis = handler_impl.vis.clone();
    let handler_attr = handler_impl.attrs.clone();
    handler_impl.sig.ident = handler_impl_name.clone();

    let asm_code = LitStr::new(&KERNEL_TRAP, proc_macro::Span::call_site().into());
    let trap_handler = quote! {
        #(#handler_attr)*
        #[naked]
        #handler_vis unsafe fn #handler_name(){
            core::arch::naked_asm!(
            #asm_code,
            size = const core::mem::size_of::<#context_type>(),
            handler = sym #handler_impl_name,
            );
            #handler_impl
        }
    };
    trap_handler.into()
}
