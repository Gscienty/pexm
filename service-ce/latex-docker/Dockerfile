FROM pandoc/latex

#RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.ustc.edu.cn/g' /etc/apk/repositories \
      RUN apk add font-noto font-linux-libertine \
      && apk add font-noto-cjk \
      && fc-cache -f \
      && tlmgr update --self \
      && tlmgr install ctex enumitem tasks titlesec adjustbox amsmath geometry fandol