# TEST-CONTRACTS-MAINT-003

公共 seam 测试使用 crate 内最小 trait doubles，不依赖 backend，也不修改 contract-testkit。红灯锁定：`all()` 在缺 repo/account/time 句柄时曾假通过；helper 名称/API 与失败传播缺证据。

绿灯后运行 contracts/contract-testkit all-targets、全部生产消费者 check/test、clippy/doc、coverage/API/版本/依赖门禁。Fake 通过只证明合同形状，不证明真实后端。
