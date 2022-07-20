use teo::app::app::App;
use teo::app::run;
use tokio::main;
use teo::core::builders::pipeline_builder::PipelineBuilder;
use teo::core::graph::Graph;
use teo::core::value::Value;


async fn make_graph() -> Graph {

    let mongo_url = match std::env::var("MONGO_URL") {
        Ok(url) => url,
        Err(_err) => "mongodb://127.0.0.1:27017/teotestserver".to_string()
    };

    Graph::new(|g| {
        g.data_source().mongodb(&mongo_url);
        g.url_prefix("/api/v1");
        g.model("Unit", |m| {
            m.field("id", |f| {
                f.primary().required().readonly().object_id().column_name("_id").auto();
            });
            m.field("num", |f| {
                f.required().f64().default(0f64).on_save(|p| {
                    p.transform(&|v: f64| async move { v + 1.0 });
                });
            });
            m.field("str", |f| {
                f.required().string().default("").on_save(|p| {
                    p.transform(&|v: String| async move { format!("~{v}~")});
                });
            });
            m.field("vec", |f| {
                f.required().vec(|v| {
                    v.required().string();
                }).default(vec!["a", "b", "qq"]);
            });
            m.field("createdAt", |f| {
                f.required().readonly().datetime().on_save(|p| {
                    p.if_p(|p| { p.is_null(); }).then_p(|p| { p.now(); });
                });
            });
            m.field("updatedAt", |f| {
                f.required().readonly().datetime().on_save(|p| {
                    p.now();
                });
            });
            m.on_created(&|object| async {
                println!("OK: this is created.");
            });
            m.on_updated(&|object| async {
                println!("OK: this is updated.");
            });
        });

        g.model("Author", |m| {
            m.field("id", |f| {
                f.primary().required().readonly().object_id().column_name("_id").auto();
            });
            m.field("name", |f| {
                f.required().string();
            });
            m.relation("articles", |r| {
               r.vec("Article").fields(vec!["id"]).references(vec!["authorId"]);
            });
            m.field("createdAt", |f| {
                f.required().readonly().datetime().on_save(|p| {
                    p.if_p(|p| { p.is_null(); }).then_p(|p| { p.now(); });
                });
            });
            m.field("updatedAt", |f| {
                f.required().readonly().datetime().on_save(|p| {
                    p.now();
                });
            });
        });

        g.model("Article", |m| {
            m.field("id", |f| {
                f.primary().required().readonly().object_id().column_name("_id").auto();
            });
            m.field("title", |f| {
                f.required().string();
            });
            m.field("published", |f| {
                f.required().bool().default(false);
            });
            m.field("authorId", |f| {
                f.required().object_id();
            });
            m.relation("author", |r| {
                r.required().object("Author").fields(vec!["authorId"]).references(vec!["id"]);
            });
            m.field("createdAt", |f| {
                f.required().readonly().datetime().on_save(|p| {
                    p.if_p(|p| { p.is_null(); }).then_p(|p| { p.now(); });
                });
            });
            m.field("updatedAt", |f| {
                f.required().readonly().datetime().on_save(|p| {
                    p.now();
                });
            });
        });

        g.model("Record", |m| {
            m.table_name("records");
            m.field("id", |f| {
                f.primary().required().readonly().object_id().column_name("_id").auto();
            });
            m.field("name", |f| {
                f.required().string().default("Bson");
            });
            m.field("age", |f| {
                f.required().u8().default(18u8);
            });
        });

        g.model("AuthCode", |m| {
            m.localized_name("短信验证码");
            m.description("用于用户登录或者修改手机号码的短信验证码。");
            m.field("id", |f| {
                f.primary().required().readonly().object_id().column_name("_id").auto();
            });
            m.field("phoneNo", |f| {
                f.localized_name("电话号码");
                f.description("接收验证码的电话号码，必填。");
                f.unique().required().string().on_set(|p| {
                    p.regex_match(r"^1\d{10}$");
                });
            });
            m.field("code", |f| {
                f.localized_name("验证码");
                f.description("是一个4位数的数字。");
                f.required().internal().string().on_save(|p| {
                    p.random_digits(4);
                });
            });
            m.field("createdAt", |f| {
                f.required().readonly().datetime().on_save(|p| {
                    p.if_p(|p| { p.is_null(); }).then_p(|p| { p.now(); });
                });
            });
            m.field("updatedAt", |f| {
                f.required().readonly().datetime().on_save(|p| {
                    p.now();
                });
            });
        });

        g.r#enum("Sex", |e| {
            e.choice("MALE", |_| {});
            e.choice("FEMALE", |_| {});
        });

        g.model("User", |m| {
            m.localized_name("用户");
            m.description("在前端平台登录的用户。");
            m.identity();
            m.field("id", |f| {
                f.primary().required().readonly().object_id().column_name("_id").auto();
            });
            m.field("authCode", |f| {
                f.localized_name("短信验证码");
                f.description("用户必须使用短信验证码来登录系统或修改个人的手机号码。");
                f.temp().optional().string();
            });
            m.field("phoneNo", |f| {
                f.localized_name("电话号码");
                f.description("用户的电话号码，必填，用作登录身份。");
                f.unique().required().string().auth_identity().on_set(|p| {
                    p.regex_match(r"^1\d{10}$");
                    // p.validate_p(|p| {
                    //     p.object_value("authCode").is_equal_p(|p| {
                    //         p.find_unique("AuthCode", {"": ""})
                    //     });
                    // })
                });
            });
            m.field("name", |f| {
                f.localized_name("用户的显示的名字");
                f.description("新用户如果没有传自己的名字，则会默认成为“用户1598899****”这样的格式。");
                f.required().string().default(|p: &mut PipelineBuilder| {
                    p.object_value("phoneNo").regex_replace(r"(.).{3}$", "****").str_prepend("用户");
                });
            });
            m.field("sex", |f| {
                f.localized_name("用户的性别");
                f.description("默认为空，只允许修改一次。");
                f.optional().r#enum("Sex").write_once();
            });
            m.relation("favorites", |r| {
                r.vec("Favorite").fields(vec!["id"]).references(vec!["userId"]);
            });
            m.relation("favoriteProducts", |r| {
                r.vec("Product").through("Favorite").local("user").foreign("product");
            });
            m.field("createdAt", |f| {
                f.required().readonly().datetime().on_save(|p| {
                    p.if_p(|p| { p.is_null(); }).then_p(|p| { p.now(); });
                });
            });
            m.field("updatedAt", |f| {
                f.required().readonly().datetime().on_save(|p| {
                    p.now();
                });
            });
        });

        g.model("Product", |m| {
            m.localized_name("产品");
            m.description("在平台中销售的产品。");
            m.field("id", |f| {
                f.primary().required().readonly().object_id().column_name("_id").auto();
            });
            m.field("name", |f| {
                f.localized_name("产品名");
                f.description("产品名称。");
                f.required().string();
            });
            m.field("description", |f| {
                f.localized_name("产品描述");
                f.description("产品描述。");
                f.optional().string();
            });
            m.relation("favorites", |r| {
                r.vec("Favorite").fields(vec!["id"]).references(vec!["productId"]);
            });
            m.relation("favoriteBy", |r| {
                r.vec("User").through("Favorite").local("product").foreign("user");
            });
            m.field("createdAt", |f| {
                f.required().readonly().datetime().on_save(|p| {
                    p.if_p(|p| { p.is_null(); }).then_p(|p| { p.now(); });
                });
            });
            m.field("updatedAt", |f| {
                f.required().readonly().datetime().on_save(|p| {
                    p.now();
                });
            });
        });

        g.model("Favorite", |m| {
            m.localized_name("用户收藏");
            m.description("用户收藏的产品，产品被收藏的用户。");
            m.field("userId", |f| {
                f.required().object_id();
            });
            m.relation("user", |r| {
                r.object("User").fields(vec!["userId"]).references(vec!["id"]);
            });
            m.field("productId", |f| {
                f.required().object_id();
            });
            m.relation("product", |r| {
                r.object("Product").fields(vec!["productId"]).references(vec!["id"]);
            });
            m.field("createdAt", |f| {
                f.required().readonly().datetime().on_save(|p| {
                    p.if_p(|p| { p.is_null(); }).then_p(|p| { p.now(); });
                });
            });
            m.field("updatedAt", |f| {
                f.required().readonly().datetime().on_save(|p| {
                    p.now();
                });
            });
            m.primary(vec!["userId", "productId"]);
        });


        g.model("Admin", |m| {
            m.localized_name("管理员");
            m.description("在管理平台登录的公司内部的管理员。");
            m.identity();
            m.field("id", |f| {
                f.primary().required().readonly().object_id().column_name("_id").auto();
            });
            m.field("email", |f| {
                f.localized_name("邮箱");
                f.description("管理员的公司邮箱。");
                f.unique().required().string().auth_identity().on_save(|p| {
                    p.email();
                });
            });
            m.field("password", |f| {
                f.writeonly().required().string().auth_by(|p: &mut PipelineBuilder| {
                    p.bcrypt_verify(|p: &mut PipelineBuilder| {
                        p.object_value("password");
                    });
                }).on_set(|p| {
                    p.length_between(8, 16).secure_password().bcrypt_salt();
                });
            });
            m.field("name", |f| {
                f.required().string();
            });
            m.field("activated", |f| {
                f.required().bool().default(true);
            });
            m.field("createdAt", |f| {
                f.required().readonly().datetime().on_save(|p| {
                    p.if_p(|p| { p.is_null(); }).then_p(|p| { p.now(); });
                });
            });
            m.field("updatedAt", |f| {
                f.required().readonly().datetime().on_save(|p| {
                    p.now();
                });
            });
        });
    }).await
}

#[main]
async fn main() -> std::io::Result<()> {
    let graph = make_graph().await;
    let app = App::new(|a| {
        a.server(|s| {
            s.jwt_secret("my secret");
        });
    });
    run(graph, app).await
}
