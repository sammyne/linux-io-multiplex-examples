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

## 2. 温馨提示
### 2.1. epoll 不能监听普通文件
示例程序参见 [cannot_poll_file.rs](./src/bin/cannot_poll_file.rs)，`epoll_ctl` 时会报错如下
```bash
epoll_ctl: Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }
```

具体原因为 `epoll_ctl` 内部会检查文件对象的 `f_op` 字段（类型为 `struct file_operations`）的 `poll` 字段是否为空。在 v6.6 的内核
中，相关函数代码片段如下

- [epoll_ctl](https://github.com/torvalds/linux/blob/v6.6/fs/eventpoll.c#L2267)
    ```c
    SYSCALL_DEFINE4(epoll_ctl, int, epfd, int, op, int, fd,
        struct epoll_event __user *, event)
    {
      struct epoll_event epds;

      if (ep_op_has_event(op) &&
          copy_from_user(&epds, event, sizeof(struct epoll_event)))
        return -EFAULT;

      return do_epoll_ctl(epfd, op, fd, &epds, false);
    }
    ```
- [do_epoll_ctl](https://github.com/torvalds/linux/blob/v6.6/fs/eventpoll.c#L2111)
    ```c
    int do_epoll_ctl(int epfd, int op, int fd, struct epoll_event *epds,
        bool nonblock)
    {
      // 此处省略其余代码

      /*
      * We have to check that the file structure underneath the file descriptor
      * the user passed to us _is_ an eventpoll file. And also we do not permit
      * adding an epoll file descriptor inside itself.
      */
      error = -EINVAL;
      if (f.file == tf.file || !is_file_epoll(f.file))
        goto error_tgt_fput;

      // 此处省略其余代码
    }
    ```
- [is_file_poll](https://github.com/torvalds/linux/blob/v6.6/fs/eventpoll.c#L338)
    ```c
    static inline int is_file_epoll(struct file *f)
    {
      return f->f_op == &eventpoll_fops;
    }
    ```

`f_op` 的具体值随文件系统类型而定。可用 `df -T 文件/目录路径` 查询某个文件/目录所在文件系统的类型，样例如下

```bash
Filesystem     Type  1K-blocks      Used Available Use% Mounted on
/dev/vdb1      ext4 1031916084 526704948 461155524  54% /github.com
```

可见，个人的文件系统类型为 ext4，对应的 `f_op` 实例（具体类型定义参见
[file_operations](https://github.com/torvalds/linux/blob/v6.6/include/linux/fs.h#L1852)）为
[ext4_file_operations](https://github.com/torvalds/linux/blob/v6.6/fs/ext4/file.c#L950) 如下，
```c
const struct file_operations ext4_file_operations = {
	.llseek		= ext4_llseek,
	.read_iter	= ext4_file_read_iter,
	.write_iter	= ext4_file_write_iter,
	.iopoll		= iocb_bio_iopoll,
	.unlocked_ioctl = ext4_ioctl,
#ifdef CONFIG_COMPAT
	.compat_ioctl	= ext4_compat_ioctl,
#endif
	.mmap		= ext4_file_mmap,
	.mmap_supported_flags = MAP_SYNC,
	.open		= ext4_file_open,
	.release	= ext4_release_file,
	.fsync		= ext4_sync_file,
	.get_unmapped_area = thp_get_unmapped_area,
	.splice_read	= ext4_file_splice_read,
	.splice_write	= iter_file_splice_write,
	.fallocate	= ext4_fallocate,
};
```

其 `poll` 字段没有设置，因此取默认值 `NULL`。

## 3. 参考文献
- [IO多路转接（复用）之epoll](https://subingwen.cn/linux/epoll/)
- [Basic non-blocking IO using epoll in Rust](https://www.zupzup.org/epoll-with-rust/index.html)
- [epoll 能监听普通文件吗？](https://cloud.tencent.com/developer/article/1835294)