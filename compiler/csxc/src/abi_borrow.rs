use crate::abi::{Contract, Scalar};

pub(crate) fn configuration(contract: &Contract) -> String {
    let count = contract.fields.len();
    let mut fields = Vec::new();
    let mut initializers = Vec::new();
    let mut methods = String::new();
    let mut offsets = String::new();
    for (index, ty) in contract.fields.values().enumerate() {
        fields.push(format!("f{index}: i64"));
        initializers.push(format!("f{index}: fields[{index}]"));
        let (kind, value) = if *ty == Scalar::Bool {
            ("bool", format!("self.f{index} == 1"))
        } else {
            ("i64", format!("self.f{index}"))
        };
        methods.push_str(&format!(
            "pub fn get_{index}(&self) -> {kind} {{ {value} }}\n"
        ));
        offsets.push_str(&format!(
            "assert!(std::mem::offset_of!(Configuration, f{index}) == {});\n",
            index * 8
        ));
    }
    format!(
        r#"
#[repr(C)]
pub struct Configuration {{ {} }}
const _: () = {{
    assert!(std::mem::size_of::<Configuration>() == {});
    assert!(std::mem::align_of::<Configuration>() == 8);
    {offsets}
}};
impl Configuration {{
    pub fn new(fields: [i64; {count}]) -> Self {{ Self {{ {} }} }}
    {methods}
}}
"#,
        fields.join(", "),
        count * 8,
        initializers.join(", ")
    )
}

pub(crate) fn borrow_reference(variable: &str, failure: &str) -> String {
    format!(
        r#"
if address.is_null() || !address.is_aligned() || address.addr() as u64 != witness {{
    {failure}
}}
// The matched compiler-generated caller holds the owning box or shared owner for this call.
let v_{variable} = unsafe {{ &*address }};
"#
    )
}

pub(crate) fn primitive_import(contract: &Contract, name: &str) -> String {
    let symbol = contract.symbol();
    let validation = if contract.result == Scalar::Bool {
        "if matches!(reply.value, 0 | 1)"
    } else {
        ""
    };
    configuration(contract)
        + &format!(
            r#"
#[repr(C)] struct Reply {{ status: i64, value: i64 }}
#[link(name = "{name}", kind = "raw-dylib")]
extern "C" {{
    #[link_name = "{symbol}"]
    fn invoke_dll(address: *const Configuration, witness: u64) -> Reply;
}}
pub fn invoke(config: &Configuration) -> Result<i64, String> {{
    let address = std::ptr::from_ref(config);
    let reply = unsafe {{ invoke_dll(address, address.addr() as u64) }};
    match reply.status {{
        0 {validation} => Ok(reply.value),
        _ => Err("CSharpX borrowed DLL call failed".into()),
    }}
}}
"#
        )
}
