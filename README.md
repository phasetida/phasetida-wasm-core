# phasetida-wasm-core
![](https://github.com/phasetida/phasetida-wasm-core/actions/workflows/build.yaml/badge.svg)  
一个简单的 Rust → WebAssembly 组件，将 [Phigros](https://www.taptap.cn/app/165287) 官方铺面文件渲染为紧凑的结构体。

## 安装
这里有两种安装方式：
### 1. 使用Github Action的构建分支
这个仓库具有Github Action的自动构建分支，运行
```bash
npm install phasetida/phasetida-wasm-core#dist
```
### 2. 手动编译
~~欸欸欸？这么信不过咱吗？~~  
1. 安装Cargo（如果已经安装了，请跳过这一步；如果没有的话，可以参考[这个教程](https://doc.rust-lang.org/cargo/getting-started/installation.html)）
2. 安装wasm-pack：
   ```bash
   cargo install wasm-pack
   ```
3. 克隆这个仓库
   ```bash
   git clone https://github.com/phasetida/phasetida-wasm-core
   ```
4. 进入仓库目录，然后执行wasm构建：
   ```bash
   wasm-pack build --target web
   ```
5. 最后，在``package.json``文件里引用仓库里的``pkg``目录就可以啦

## 使用
这个WASM组件会读取``window.inputBuffer``，``window.inputBufferLength``，``window.outputBuffer``和``window.outputBufferLength``，所以请准备好这些缓冲区和缓冲区长度。  
以下为示例代码
```javascript
import init, { pre_draw, load_level } from "phasetida_wasm_core.js";

const OUTPUT_BUFFER_LENGTH = 65536;
const outputBufferRaw = new ArrayBuffer(
  OUTPUT_BUFFER_LENGTH * Uint8Array.BYTES_PER_ELEMENT
);
const outputBuffer = new Uint8Array(outputBufferRaw);
const outputBufferLength = outputBuffer.length;
window.outputBuffer = outputBuffer;
window.outputBufferLength = outputBufferLength;

const INPUT_BUFFER_LENGTH = 1024;
const inputBufferRaw = new ArrayBuffer(
  INPUT_BUFFER_LENGTH * Uint8Array.BYTES_PER_ELEMENT
);
const inputBuffer = new Uint8Array(inputBufferRaw);
const inputBufferLength = inputBuffer.length;
window.inputBuffer = inputBuffer;
window.inputBufferLength = inputBufferLength;

await init();
//...
```
调用load_level时，WASM会接受参数的JSON字符串，并载入铺面文件。  
调用pre_draw时，WASM会从``window.inputBuffer``读取输入数据，并将绘制结构化数据写入``window.outputBuffer``，具体结构化数据格式，请见``src/renders.rs``以及``src/input.rs``

## 注意
本仓库为实验性玩具项目，所以文档十分潦草。  
由于Android WebView对共享缓冲区的限制，本仓库不使用共享缓冲区作为数据交换的方式！  