<div align="center">

# VCMP Python Plugin Link

这是一个用于VCMP的Python插件，可以方便简洁的在不同的Python版本之间切换，并且可以方便的安装Python插件。

</div>

## Usage

1. 下载并解压文件，将`vcmp_python_plugin`文件夹放入`plugins`文件夹中
2. 在 `server.cfg` 中添加以下内容：

```cfg
python_plugins_dir ./libraries
python_filename_format python04rel64rspyo3py{py_version}
```

## 适用于项目

[VCMP Python Plugin](https://github.com/tianxiu2b2t/vcmp-python-plugin/)