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

        g.model("Creator", |m| {
            m.field("id", |f| {
                f.primary().required().readonly().object_id().column_name("_id").auto();
            });
            m.field("name", |f| {
                f.required().string();
            });
            m.relation("packages", |r| {
                r.vec("Package").fields(vec!["id"]).references(vec!["creatorId"]);
            });
            m.relation("editions", |r| {
                r.vec("Edition").fields(vec!["id"]).references(vec!["creatorId"]);
            });
        });

        g.model("Package", |m| {
            m.field("id", |f| {
                f.primary().required().readonly().object_id().column_name("_id").auto();
            });
            m.field("name", |f| {
                f.required().string();
            });
            m.relation("editions", |r| {
                r.vec("Edition").fields(vec!["id"]).references(vec!["packageId"]);
            });
            m.field("creatorId", |f| {
                f.required().object_id();
            });
            m.relation("creator", |r| {
               r.object("Creator").fields(vec!["creatorId"]).references(vec!["id"]);
            });
        });

        g.model("Edition", |m| {
            m.field("id", |f| {
                f.primary().required().readonly().object_id().column_name("_id").auto();
            });
            m.field("name", |f| {
                f.required().string();
            });
            m.field("packageId", |f| {
                f.required().object_id();
            });
            m.relation("package", |r| {
               r.object("Package").fields(vec!["packageId"]).references(vec!["id"]);
            });
            m.field("creatorId", |f| {
                f.required().object_id();
            });
            m.relation("creator", |r| {
                r.object("Creator").fields(vec!["creatorId"]).references(vec!["id"]);
            });
        });

        g.model("Unit", |m| {
            m.field("id", |f| {
                f.primary().required().readonly().object_id().column_name("_id").auto();
            });
            m.field("bool", |f| {
                f.optional().bool();
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
            m.on_created(&|object| async move {
                let string: String = object.get("str")?;
                let vec: Vec<String> = object.get("vec")?;
                let num: f64 = object.get("num")?;
                let bool: Option<bool> = object.get("bool")?;
                println!("created: {string} {vec:?} {num} {bool:?}");
                object.set("num", 25)?;
                object.save().await?;
                Ok(())
            });
            m.on_updated(&|object| async move {
                let string: String = object.get("str")?;
                let vec: Vec<String> = object.get("vec")?;
                let num: f64 = object.get("num")?;
                let bool: Option<bool> = object.get("bool")?;
                println!("updated: {string} {vec:?} {num} {bool:?}");
                object.set("num", 80)?;
                object.save().await?;
                Ok(())
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
            m.localized_name("???????????????");
            m.description("???????????????????????????????????????????????????????????????");
            m.field("id", |f| {
                f.primary().required().readonly().object_id().column_name("_id").auto();
            });
            m.field("phoneNo", |f| {
                f.localized_name("????????????");
                f.description("??????????????????????????????????????????");
                f.unique().required().string().on_set(|p| {
                    p.regex_match(r"^1\d{10}$");
                });
            });
            m.field("code", |f| {
                f.localized_name("?????????");
                f.description("?????????4??????????????????");
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
            m.localized_name("??????");
            m.description("?????????????????????????????????");
            m.identity();
            m.field("id", |f| {
                f.primary().required().readonly().object_id().column_name("_id").auto();
            });
            m.field("authCode", |f| {
                f.localized_name("???????????????");
                f.description("?????????????????????????????????????????????????????????????????????????????????");
                f.temp().optional().string();
            });
            m.field("phoneNo", |f| {
                f.localized_name("????????????");
                f.description("??????????????????????????????????????????????????????");
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
                f.localized_name("????????????????????????");
                f.description("?????????????????????????????????????????????????????????????????????1598899****?????????????????????");
                f.required().string().default(|p: &mut PipelineBuilder| {
                    p.object_value("phoneNo").regex_replace(r"(.).{3}$", "****").str_prepend("??????");
                });
            });
            m.field("sex", |f| {
                f.localized_name("???????????????");
                f.description("???????????????????????????????????????");
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
            m.localized_name("??????");
            m.description("??????????????????????????????");
            m.field("id", |f| {
                f.primary().required().readonly().object_id().column_name("_id").auto();
            });
            m.field("name", |f| {
                f.localized_name("?????????");
                f.description("???????????????");
                f.required().string();
            });
            m.field("description", |f| {
                f.localized_name("????????????");
                f.description("???????????????");
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
            m.localized_name("????????????");
            m.description("???????????????????????????????????????????????????");
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
            m.localized_name("?????????");
            m.description("???????????????????????????????????????????????????");
            m.identity();
            m.field("id", |f| {
                f.primary().required().readonly().object_id().column_name("_id").auto();
            });
            m.field("email", |f| {
                f.localized_name("??????");
                f.description("???????????????????????????");
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
            s.bind(("0.0.0.0", 5300u16));
        });
    });
    run(graph, app).await
}
