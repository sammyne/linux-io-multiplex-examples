# epoll

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
feedback = 'hello world'
```

## 2. 核心函数接口

### 2.1. `epoll_create`

```c
/**
 * @brief 创建一个红黑树模型的实例，用于管理待检测的文件描述符的集合。
 * 
 * @param size 参数 size：在 Linux 内核 2.6.8 版本以后，这个参数是被忽略的，只需要指定一个大于 0 的数值就可以了。 返回值：
 * @return int 失败：返回 -1；- 成功：返回一个有效的文件描述符，通过这个文件描述符就可以访问创建的 epoll 实例了
 */
int epoll_create(int size);
```

### 2.2. `epoll_ctl`
```c

// 联合体, 多个变量共用同一块内存
typedef union epoll_data {
  void *ptr;
  int fd;  // 通常情况下使用这个成员, 和 epoll_ctl 的第三个参数相同即可
  uint32_t u32;
  uint64_t u64;
} epoll_data_t;

/**
 * @brief 指定检测这个文件描述符的什么事件
 */
struct epoll_event {
  uint32_t events;    //< 委托epoll检测的事件，枚举值如下
                      //<   - EPOLLIN：读事件, 接收数据，检测读缓冲区，如果有数据该文件描述符就绪
                      //<   - EPOLLOUT：写事件, 发送数据，检测写缓冲区，如果可写该文件描述符就绪
                      //<   - EPOLLERR：异常事件
                      //<
  epoll_data_t data;  //< 用户数据变量，这是一个联合体类型，通常情况下使用里边的 fd 成员，用于存储待检测的文件描述符的值，
                      //< 在调用epoll_wait()函数的时候这个值会被传出。
                      //<
};

/**
 * @brief 管理红黑树实例上的节点，可以进行添加、删除、修改操作。
 *
 * @param epfd epoll_create() 函数的返回值，通过这个参数找到 epoll 实例
 * @param op 一个枚举值，控制通过该函数执行什么操作
            - EPOLL_CTL_ADD：往 epoll 模型中添加新的节点
            - EPOLL_CTL_MOD：修改 epoll 模型中已经存在的节点
            - EPOLL_CTL_DEL：删除 epoll 模型中的指定的节点
 * @param fd 文件描述符，即要添加/修改/删除的文件描述符
 * @param event epoll 事件，用来修饰第三个参数对应的文件描述符的，指定检测这个文件描述符的什么事件
 * @return int 失败：返回 -1；成功：返回 0
 */
int epoll_ctl(int epfd, int op, int fd, struct epoll_event *event);
```

### 2.3. `epoll_wait`
```c
/**
 * @brief 检测创建的 epoll 实例中有没有就绪的文件描述符。
 * 
 * @param epfd epoll_create() 函数的返回值, 通过这个参数找到 epoll 实例
 * @param events 传出参数, 是一个结构体数组的地址, 里边存储了已就绪的文件描述符的信息
 * @param maxevents 修饰第二个参数, 结构体数组的容量（元素个数）
 * @param timeout 如果检测的 epoll 实例中没有已就绪的文件描述符，该函数阻塞的时长, 单位 ms（毫秒）
 * 								- 0：函数不阻塞，不管 epoll 实例中有没有就绪的文件描述符，函数被调用后都直接返回；
 * 								- 大于 0：如果epoll实例中没有已就绪的文件描述符，函数阻塞对应的毫秒数再返回；
 * 								- -1：函数一直阻塞，直到epoll实例中有已就绪的文件描述符之后才解除阻塞
 * @return int 成功：
									- 等于 0：函数是阻塞被强制解除了, 没有检测到满足条件的文件描述符；
									- 大于 0：检测到的已就绪的文件描述符的总个数；
								失败：返回-1
 */
int epoll_wait(int epfd, struct epoll_event * events, int maxevents, int timeout);
```

## 2. 参考文献
- [IO多路转接（复用）之epoll](https://subingwen.cn/linux/epoll/)
- [Basic non-blocking IO using epoll in Rust](https://www.zupzup.org/epoll-with-rust/index.html)
