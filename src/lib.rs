use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{WebGlRenderingContext, WebGlShader, WebGlProgram};

/// 为传入 canvas_id 创建一个 webGL 实例并返回
/// # Arguments
/// * `canvas_id` - html 中 canvas 标签的 id
pub fn init_webgl_context(canvas_id: &str) -> Result<WebGlRenderingContext, JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id(canvas_id).unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
    let gl: WebGlRenderingContext = canvas
        .get_context("webgl")?
        .unwrap()
        .dyn_into::<WebGlRenderingContext>()
        .unwrap();

    // webgl 大小即为 canvas 大小
    gl.viewport(
        0,
        0,
        canvas.width().try_into().unwrap(),
        canvas.height().try_into().unwrap(),
    );

    Ok(gl)
}

/// 创建一个着色器
/// # Arguments
/// * `gl` - webGl 上下文
/// * `shader_type` 要创建的着色器类型
/// * `source` 着色器源码
pub fn create_shader(
    gl: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, JsValue> {
    // 创建一个新的着色器对象。如果创建失败（返回 None），使用 JsValue::from_str 返回错误消息
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| JsValue::from_str("Unable to create shader object"))?;

    // 设置着色器的源码
    gl.shader_source(&shader, source);
    // 编译着色器
    gl.compile_shader(&shader);

    // 使用 get_shader_parameter 和 WebGlRenderingContext::COMPILE_STATUS 检索编译状态来检查着色器编译是否成功
    if gl
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        /*
         * 使用 使用 JsValue::from_str 处理返回的错误消息
         * 使用 gl.get_shader_info_log 检索着色器信息日志
         * 如果有日志，返回日志，否则返回一条通用错误消息
         */
        Err(JsValue::from_str(
            &gl.get_shader_info_log(&shader)
                .unwrap_or_else(|| "Unknown error creating shader".into()),
        ))
    }
}

/// 给传入 webgl 上下文创建一个 webgl 程序，如果创建成功则返回该程序
/// * `gl` - webGl 上下文
pub fn setup_shaders(gl: &WebGlRenderingContext) -> Result<WebGlProgram, JsValue> {
    /*
     * 创建一个顶点着色器的 GLSL 源码
     * 接受单个属性坐标 coordinates ，并将顶点的位置 gl_Position 设置为这些坐标，其中 w 组件为 1.0
     */
    let vertex_shader_source = "
        attribute vec3 coordinates;
        void main(void) {
            gl_Position = vec4(coordinates, 1.0);
        }
        ";

    /*
     * 创建一个片元着色器的 GLSL 源码
     * 将每个像素 gl_FragColor 的颜色设置为统一 fragColor 的值
     */
    let fragment_shader_source = "
        precision mediump float;
        uniform vec4 fragColor;
        void main(void) {
            gl_FragColor = fragColor;
        }
        ";

    // 设置顶点着色器和片元着色器的源码
    let vertex_shader = create_shader(
        &gl,
        WebGlRenderingContext::VERTEX_SHADER,
        vertex_shader_source,
    )
    .unwrap();
    let fragment_shader = create_shader(
        &gl,
        WebGlRenderingContext::FRAGMENT_SHADER,
        fragment_shader_source,
    )
    .unwrap();

    // 创建着色器程序
    let shader_program = gl.create_program().unwrap();

    // 将顶点着色器和片元着色器附加到着色器程序中
    gl.attach_shader(&shader_program, &vertex_shader);
    gl.attach_shader(&shader_program, &fragment_shader);

    // 将着色器程序链接到 webgl 上下文中
    gl.link_program(&shader_program);

    // 使用 get_program_parameter 和 WebGlRenderingContext::LINK_STATUS 确定程序是否链接成功
    if gl
        .get_program_parameter(&shader_program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        // 如果链接成功，将其设置为 WebGL 上下文的活动程序
        gl.use_program(Some(&shader_program));
        // 返回该程序
        Ok(shader_program)
    } else {
        return Err(JsValue::from_str(
            &gl.get_program_info_log(&shader_program)
                .unwrap_or_else(|| "Unknown error linking program".into()),
        ));
    }
}
