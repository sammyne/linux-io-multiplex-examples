# select

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
struct timeval {
    long    tv_sec;         /* seconds */
    long    tv_usec;        /* microseconds */
};

/**
 * @brief 
 * 
 * @param nfds 委托内核检测的这三个集合中最大的文件描述符 + 1。内核需要线性遍历这些集合中的文件描述符，这个值是循环结束的条件。
 *              在 Window 中这个参数是无效的，指定为 -1 即可
 * @param[in,out] readfds 文件描述符的集合, 内核只检测这个集合中文件描述符对应的读缓冲区。读集合一般情况下都是需要检测的，
 *                这样才知道通过哪个文件描述符接收数据
 * @param[in,out] writefds 文件描述符的集合, 内核只检测这个集合中文件描述符对应的写缓冲区。如果不需要使用这个参数可以指定为 NULL
 * @param[in,out] exceptfds 文件描述符的集合, 内核检测集合中文件描述符是否有异常状态。如果不需要使用这个参数可以指定为 NULL
 * @param timeout 超时时长，用来强制解除函数阻塞
 *                - NULL：函数检测不到就绪的文件描述符会一直阻塞；
 *                - 等待固定时长（秒）：函数检测不到就绪的文件描述符，在指定时长之后强制解除阻塞，函数返回 0；
 *                - 不等待：函数不会阻塞，直接将该参数对应的结构体初始化为0即可。
 * @return int  - 大于 0：成功，返回集合中已就绪的文件描述符的总个数；
 *              - 等于-1：函数调用失败；
 *              - 等于 0：超时，没有检测到就绪的文件描述符
 */
int select(int nfds, fd_set *readfds, fd_set *writefds, fd_set *exceptfds, struct timeval * timeout);

// 将文件描述符fd从set集合中删除 == 将fd对应的标志位设置为0        
void FD_CLR(int fd, fd_set *set);

// 判断文件描述符fd是否在set集合中 == 读一下fd对应的标志位到底是0还是1
int  FD_ISSET(int fd, fd_set *set);

// 将文件描述符fd添加到set集合中 == 将fd对应的标志位设置为1
void FD_SET(int fd, fd_set *set);

// 将set集合中, 所有文件文件描述符对应的标志位设置为0, 集合中没有添加任何文件描述符
void FD_ZERO(fd_set *set);
```

## 3. 参考文献
- [IO多路转接（复用）之select](https://subingwen.cn/linux/select/)
