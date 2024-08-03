use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader};

/// 为传入 canvas_id 创建一个 WebGL 实例并返回
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

    // WebGL 大小即为 canvas 大小
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
/// * `gl` - WebGL 上下文
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

/// 给传入 WebGL 上下文创建一个 WebGL 程序，如果创建成功则返回该程序
/// * `gl` - WebGL 上下文
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

/// 将顶点列表放入 WebGL 的缓冲区，并设置到 WebGL 程序中
/// * `gl` - WebGL 上下文
/// * `vertices` - 顶点列表
/// * `shader_program` - WebGL 程序
pub fn setup_vertices(gl: &WebGlRenderingContext, vertices: &[f32], shader_program: &WebGlProgram) {
    // 创建顶点数组，这会直接扰乱计算机内存，所以使用 unsafe
    let vertices_array = unsafe { js_sys::Float32Array::view(&vertices) };
    // 创建和使用缓冲区，它就像一个临时存储空间，用于放置 WebGL 将使用的数据
    let vertex_buffer = gl.create_buffer().unwrap();

    // 将缓冲绑定到 WebGL 内部的 ARRAY_BUFFER 绑定点上
    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
    /*
     * 将数据 vertices_array 存放到 ARRAY_BUFFER 绑定点的缓冲中
     * 第三个参数提示 WebGL 将怎么使用这些数据，STATIC_DRAW 提示 WebGL 我们不会经常改变这些数据，WebGL 会根据提示做出一些优化
     */
    gl.buffer_data_with_array_buffer_view(
        WebGlRenderingContext::ARRAY_BUFFER,
        &vertices_array,
        WebGlRenderingContext::STATIC_DRAW,
    );

    // 找到 coordinates 属性的位置（这是 setup_shaders 的 vertex_shader_source 中使用的属性）
    let coordinates_location = gl.get_attrib_location(&shader_program, "coordinates");

    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
    // 告诉 WebGL 如何从 ARRAY_BUFFER 中读取数据
    gl.vertex_attrib_pointer_with_i32(
        coordinates_location as u32,  // 所要设置读取方式的属性
        3,                            // 每次迭代运行提取三个单位数据（x、y、z）
        WebGlRenderingContext::FLOAT, // 每个单位的数据类型是 32 位浮点型
        false,                        // 不需要归一化数据
        0, // 0 = 移动单位数量 * 每个单位占用内存（sizeof(type)）每次迭代运行运动多少内存到下一个数据开始点
        0, // 从缓冲起始位置开始读取
    );
    // 启动这个属性的设置
    gl.enable_vertex_attrib_array(coordinates_location as u32);
}

/// 绘制一个三角形并返回 WebGL 上下文
/// * `canvas_id` - 绘制目标 canvas id
/// * `selected_color` - 可选，绘制三角形的颜色
#[wasm_bindgen]
pub fn draw_triangle(
    canvas_id: &str,
    selected_color: Option<Vec<f32>>,
) -> Result<WebGlRenderingContext, JsValue> {
    let gl: WebGlRenderingContext = init_webgl_context(canvas_id).unwrap();
    let shader_program: WebGlProgram = setup_shaders(&gl).unwrap();

    // 定义三角形的点, 每个点都有 x、y、z 三个值
    let vertices: [f32; 9] = [
        0.0, 1.0, 0.0, // top
        -1.0, -1.0, 0.0, // bottom left
        1.0, -1.0, 0.0, // bottom right
    ];
    setup_vertices(&gl, &vertices, &shader_program);

    // 初始化颜色，处理可选情况
    let color = selected_color.unwrap_or(vec![1.0, 0.0, 0.0, 1.0]);
    // 找到 setup_shaders 的 fragment_shader_source 中使用的 fragColor 属性
    let color_location = gl
        .get_uniform_location(&shader_program, "fragColor")
        .unwrap();
    // 给属性设置颜色
    gl.uniform4fv_with_f32_array(Some(&color_location), &color);

    // 绘制三角形
    gl.draw_arrays(
        WebGlRenderingContext::TRIANGLES, // 三角形
        0, // 从第一个顶点开始
        (vertices.len() / 3) as i32, // 每个顶点由三个值定义
    );

    Ok(gl)
}









