# Sui Transfer

批量生成钱包，并交易一笔数据上链


## 安装

https://www.rust-lang.org/tools/install

## 创建钱包
https://www.defidaonews.com/article/6783180


## 使用

```
cargo run $phrase $object_id $count

phrase 转账的的助记词
object_id https://explorer.devnet.sui.io/ 这个上面去你的地址里面，复制一个
count  生成的钱包数量

例如 
cargo run "medal hotel ..." 0x1a656e8a9d75b2508  1
```

