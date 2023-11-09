# poll

## 1. 快速开始
### 1.1. 启动服务端
```bash
cargo run
```

### 1.2. 启动客户端发送请求

> 温馨提示：此步骤需要在另一终端执行。

```bash
cargo run --bin client
```

样例输出如下

```bash
hello world
```

> 注意：客户端程序不会退出，需要 `CTRL` + `C` 关闭其进程。

## 2. 核心函数接口

```c
/**
 * @brief 每个委托 poll 检测的 fd 都对应这样一个结构体
 */
struct pollfd {
  int fd;         //< 委托内核检测的文件描述符
  short events;   //< 委托内核检测文件描述符的事件（输入、输出或错误）,每个时间可以取多个值
  short revents;  //< 文件描述符实际发生的事件 -> 传出，数据由内核写入，存储内核检测之后的结果
};

/**
 * @brief
 *
 * @param fds 这是一个 struct pollfd 类型的数组，里边存储了待检测的文件描述符的信息

                          nfds : 这是第一个参数数组中最后一个有效元素的下标 +
                                 1（也可以指定参数1数组的元素总个数）
 * @param nfds fds 数组最后一个有效元素的下标 + 1
 * @param timeout 指定函数的阻塞时长
 * @return int      -1：一直阻塞，直到检测的集合中有就绪的文件描述符（有事件产生）解除阻塞；
 *                  0：不阻塞，不管检测集合中有没有已就绪的文件描述符，函数马上返回；
 *             大于 0：阻塞指定的毫秒（ms）数之后，解除阻塞
 */
int poll(struct pollfd *fds, nfds_t nfds, int timeout);
```

## 3. 参考文献
- [IO多路转接（复用）之poll](https://subingwen.cn/linux/poll/)
