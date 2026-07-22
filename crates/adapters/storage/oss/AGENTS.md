# ossx

- 默认真实路径为 `OssClient`（OSS Signature V1）并实现 `contracts::ObjectStore`；内存实现仅在 `scaffold` feature。
- 允许在明确证据范围内声明 HTTPS put/get/delete 与有界 multipart；禁止外推为 STS、lifecycle、TB 流式对象或 package stable。
- 凭据只通过环境或安全 live runner 注入，禁止写入源码、日志和仓库。
